use anyhow::Result;
use clap::Parser;
use geo::{Contains, EuclideanLength, LineString};
use geojson::{de::deserialize_geometry, Feature, GeoJson, Geometry};
use graph::{Graph, Mode, Timer};
use serde::Deserialize;

#[derive(Parser)]
struct Args {
    /// Path to a .osm.pbf or .xml file
    #[arg(long)]
    osm: String,

    /// Path to a .geojson file with routes to snap and buffer
    #[arg(long)]
    routes: String,
}

fn main() -> Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let args = Args::parse();
    let mut timer = Timer::new("snap routes", None);

    timer.push("build graph");
    let modify_roads = |_roads: &mut Vec<graph::Road>| {};
    let graph = Graph::new(
        &fs_err::read(&args.osm)?,
        &mut utils::osm2graph::NullReader,
        modify_roads,
        &mut timer,
    )?;
    timer.pop();

    timer.step("snap routes");
    let mut features = Vec::new();
    let mut routes = Vec::new();
    let mode = Mode::Bicycle;
    let mut errors = 0;
    let mut len_pcts = Vec::new();
    for mut input in geojson::de::deserialize_feature_collection_str_to_vec::<GeoJsonLineString>(
        &fs_err::read_to_string(&args.routes)?,
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
        match graph.snap_route(&input.geometry, mode) {
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
