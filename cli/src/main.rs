use std::fs::File;
use std::io::BufWriter;
use std::time::Duration;

use anyhow::{bail, Result};
use backend::MapModel;
use chrono::NaiveTime;
use clap::{Parser, Subcommand};
use geo::{Contains, Coord, Euclidean, Length, LineString, Point};
use geojson::{de::deserialize_geometry, Feature, GeoJson, Geometry};
use graph::{Direction, Graph, GtfsModel, ProfileID, Route, Timer};
use serde::{Deserialize, Serialize};

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
        /// Path to a model.bin file
        #[arg(long)]
        model: String,

        /// Path to a .geojson file with routes to snap and buffer
        #[arg(long)]
        routes: String,

        #[arg(long, default_value_t = 1)]
        buffer_mins: u64,
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
        Command::SnapTest {
            model,
            routes,
            buffer_mins,
        } => snap_test(model, routes, Duration::from_secs(buffer_mins * 60)),
    }
}

fn snap_test(model_path: String, routes_path: String, limit: Duration) -> Result<()> {
    let mut timer = Timer::new("snap routes", None);

    timer.step("load model");
    let mut model: MapModel = bincode::deserialize(&fs_err::read(&model_path)?)?;

    timer.step("prepare distance-based routing");
    let distance_profile = model.graph_mut().add_profile(
        "distance".to_string(),
        Box::new(|_, linestring| {
            (
                Direction::Both,
                Duration::from_secs_f64(Euclidean.length(linestring)),
            )
        }),
    );

    let graph = model.graph();

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
        input_f.set_property("waypoints", serde_json::to_value(&input.waypoints)?);

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

        match snap(&input, graph, distance_profile) {
            Ok(route) => {
                let output = route.linestring(graph);
                let mut f = Feature::from(Geometry::from(&graph.mercator.to_wgs84(&output)));
                f.set_property("kind", "snapped");

                if let Some((len_pct, dist_between_equiv_pts)) =
                    graph::snap::score_similarity(&input.geometry, &output)
                {
                    println!("len_pct {len_pct}, dist_between_equiv_pts {dist_between_equiv_pts}");

                    if len_pct > 10.0 {
                        println!(
                            "  Skipping; this is too likely wrong. Input length {}, output {}",
                            Euclidean.length(&input.geometry),
                            Euclidean.length(&output)
                        );
                        errors += 1;
                        continue;
                    }

                    f.set_property("len_pct", len_pct);
                    f.set_property("dist_between_equiv_pts", dist_between_equiv_pts);
                    len_pcts.push(len_pct);
                } else {
                    println!("scoring broke! output len is {}", Euclidean.length(&output));
                }

                // Only show inputs successfully snapped (to conveniently clip, for now)
                features.push(input_f);
                features.push(f);
                routes.push(route);

                // Also turn all of the input waypoints into points, for even easier debugging
                for (idx, waypt) in input.waypoints.into_iter().enumerate() {
                    let mut f = Feature::from(Geometry::from(&Point::new(waypt.lon, waypt.lat)));
                    f.set_property("kind", "waypoint");
                    f.set_property("snapped", waypt.snapped);
                    f.set_property("idx", idx);
                    features.push(f);
                }
            }
            Err(_err) => {
                errors += 1;
            }
        }
    }

    let num_routes = routes.len();
    timer.step("buffer around routes");
    let start_time = NaiveTime::from_hms_opt(7, 0, 0).unwrap();
    fs_err::write(
        "buffered.geojson",
        model.buffer_routes(routes, graph.profile_names["bicycle"], start_time, limit)?,
    )?;

    timer.done();

    println!("Snapped {num_routes} routes, failed on {errors}");
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
    #[serde(default)]
    waypoints: Vec<Waypoint>,
}

#[derive(Serialize, Deserialize)]
struct Waypoint {
    lon: f64,
    lat: f64,
    snapped: bool,
}

fn snap(input: &GeoJsonLineString, graph: &Graph, profile: ProfileID) -> Result<Route> {
    // Use waypoints if they're all snapped
    if !input.waypoints.is_empty() && input.waypoints.iter().all(|waypt| waypt.snapped) {
        let pts: Vec<Coord> = input
            .waypoints
            .iter()
            .map(|waypt| {
                graph.mercator.pt_to_mercator(Coord {
                    x: waypt.lon,
                    y: waypt.lat,
                })
            })
            .collect();
        if false && pts.len() != 2 {
            bail!("TODO, skip route with more than two waypoints");
        }
        let mut routes = Vec::new();
        for pair in pts.windows(2) {
            routes.push(graph.snap_route(&LineString::new(pair.to_vec()), profile)?);
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

    bail!("TODO, skip without all snapped waypoints");
}
