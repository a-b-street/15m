use std::fs::File;
use std::io::BufWriter;

use anyhow::Result;
use backend::MapModel;
use clap::{Parser, Subcommand};
use geo::{Contains, Coord, EuclideanLength, LineString};
use geojson::{de::deserialize_geometry, Feature, GeoJson, Geometry};
use graph::{Graph, GtfsModel, Mode, Route, Timer};
use serde::Deserialize;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    BuildGraph {
        osm_path: String,
    },
    BuildGTFS {
        gtfs_dir: String,
    },
    SnapTest {
        /// Path to a .osm.pbf or .xml file
        #[arg(long)]
        osm: String,

        /// Path to a .geojson file with routes to snap and buffer
        #[arg(long)]
        routes: String,
    },
}

// TODO Don't need tokio multithreading, but fighting config to get single working
/// This is a CLI tool to build a MapModel file, for later use in the web app or CLI. This gives a
/// perf benefit (faster to load a pre-built graph), but manually managing these prebuilt files as
/// the format changes is tedious. That's why, unlike in A/B Street, this'll just be for manual
/// testing for now.
#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let args = Args::parse();

    match args.command {
        Command::BuildGraph { osm_path } => {
            let mut timer = Timer::new("build model", None);
            let osm_bytes = std::fs::read(&osm_path)?;
            let model = MapModel::create(
                &osm_bytes,
                // TODO Hardcoded, or could we read from local files at least?
                Some("https://assets.od2net.org/gtfs.gmd".to_string()),
                Some("https://assets.od2net.org/population.fgb".to_string()),
                &mut timer,
            )
            .await?;

            timer.step("Writing");
            let writer = BufWriter::new(File::create("model.bin")?);
            bincode::serialize_into(writer, &model)?;

            timer.done();
            Ok(())
        }
        Command::BuildGTFS { gtfs_dir } => {
            let mut timer = Timer::new("build geomedea from gtfs", None);
            timer.step("parse GTFS");
            let model = GtfsModel::parse(&gtfs_dir, None)?;
            timer.step("turn into geomedea");
            model.to_geomedea("gtfs.gmd")?;
            timer.done();
            Ok(())
        }
        Command::SnapTest { osm, routes } => snap_test(osm, routes),
    }
}

fn snap_test(osm: String, routes_path: String) -> Result<()> {
    let mut timer = Timer::new("snap routes", None);

    timer.push("build graph");
    let modify_roads = |_roads: &mut Vec<graph::Road>| {};
    let graph = Graph::new(
        &fs_err::read(&osm)?,
        &mut utils::osm2graph::NullReader,
        modify_roads,
        &mut timer,
    )?;
    timer.pop();

    timer.step("snap routes");
    let mut features = Vec::new();
    let mut routes = Vec::new();
    let mut errors = 0;
    let mut len_pcts = Vec::new();
    for mut input in geojson::de::deserialize_feature_collection_str_to_vec::<GeoJsonLineString>(
        &fs_err::read_to_string(&routes_path)?,
    )? {
        let mut input_f = Feature::from(Geometry::from(&input.geometry));
        input_f.set_property("kind", "input");

        // Filter out inputs obviously far away, just based on endpoints
        if !graph.mercator.wgs84_bounds.contains(&input.geometry.0[0])
            || !graph
                .mercator
                .wgs84_bounds
                .contains(input.geometry.0.last().unwrap())
        {
            errors += 1;
            continue;
        }

        graph.mercator.to_mercator_in_place(&mut input.geometry);

        match snap(&input, &graph) {
            Ok(route) => {
                let output = route.linestring(&graph);
                let mut f = Feature::from(Geometry::from(&graph.mercator.to_wgs84(&output)));
                f.set_property("kind", "snapped");

                if let Some((len_pct, dist_between_equiv_pts)) =
                    graph::snap::score_similarity(&input.geometry, &output)
                {
                    println!("len_pct {len_pct}, dist_between_equiv_pts {dist_between_equiv_pts}");

                    if len_pct > 10.0 {
                        println!(
                            "  Skipping; this is too likely wrong. Input length {}, output {}",
                            input.geometry.euclidean_length(),
                            output.euclidean_length()
                        );
                        errors += 1;
                        continue;
                    }

                    f.set_property("len_pct", len_pct);
                    f.set_property("dist_between_equiv_pts", dist_between_equiv_pts);
                    len_pcts.push(len_pct);
                } else {
                    println!("scoring broke! output len is {}", output.euclidean_length());
                }

                // Only show inputs successfully snapped (to conveniently clip, for now)
                features.push(input_f);
                features.push(f);
                routes.push(route);
            }
            Err(_err) => {
                errors += 1;
            }
        }
    }

    timer.done();

    println!("Snapped {} routes, failed on {}", routes.len(), errors);
    println!(
        "Average len_pct is {}",
        len_pcts.iter().cloned().sum::<f64>() / (len_pcts.len() as f64)
    );

    fs_err::write(
        "snapped.geojson",
        serde_json::to_string(&GeoJson::from(features))?,
    )?;

    Ok(())
}

#[derive(Deserialize)]
struct GeoJsonLineString {
    #[serde(deserialize_with = "deserialize_geometry")]
    geometry: LineString,
    waypoints: Option<Vec<Waypoint>>,
}

#[derive(Deserialize)]
struct Waypoint {
    lon: f64,
    lat: f64,
    snapped: bool,
}

fn snap(input: &GeoJsonLineString, graph: &Graph) -> Result<Route> {
    let mode = Mode::Bicycle;

    // Try to use waypoints?
    if input
        .waypoints
        .as_ref()
        .map(|waypts| waypts.iter().all(|waypt| waypt.snapped))
        .unwrap_or(false)
    {
        let pts: Vec<Coord> = input
            .waypoints
            .as_ref()
            .unwrap()
            .iter()
            .map(|waypt| {
                graph.mercator.pt_to_mercator(Coord {
                    x: waypt.lon,
                    y: waypt.lat,
                })
            })
            .collect();
        let mut routes = Vec::new();
        for pair in pts.windows(2) {
            let route = graph.snap_route(&LineString::new(pair.to_vec()), mode)?;
            routes.push(route);
        }

        // TODO Naively concatenate
        let mut steps = Vec::new();
        for route in &mut routes {
            steps.extend(std::mem::take(&mut route.steps));
        }
        return Ok(Route {
            start: routes[0].start,
            end: routes.last().unwrap().end,
            steps,
        });
    }

    graph.snap_route(&input.geometry, mode)
}
