use anyhow::Result;
use clap::Parser;
use geo::LineString;
use geojson::de::deserialize_geometry;
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

    let modify_roads = |_roads: &mut Vec<graph::Road>| {};
    let graph = Graph::new(
        &fs_err::read(&args.osm)?,
        &mut utils::osm2graph::NullReader,
        modify_roads,
        &mut timer,
    )?;

    let mode = Mode::Bicycle;
    for mut input in geojson::de::deserialize_feature_collection_str_to_vec::<GeoJsonLineString>(
        &fs_err::read_to_string(&args.routes)?,
    )? {
        graph.mercator.to_mercator_in_place(&mut input.geometry);
        graph.snap_route(&input.geometry, mode)?;
    }

    Ok(())
}

#[derive(Deserialize)]
struct GeoJsonLineString {
    #[serde(deserialize_with = "deserialize_geometry")]
    geometry: LineString,
}
