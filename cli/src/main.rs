use anyhow::Result;
use clap::Parser;
use geo::LineString;
use geojson::de::deserialize_geometry;
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

    for mut input in geojson::de::deserialize_feature_collection_str_to_vec::<GeoJsonLineString>(
        &fs_err::read_to_string(&args.routes)?,
    )? {
        /*self.graph
            .mercator
            .to_mercator_in_place(&mut input.geometry);
        routes.push(
            self.graph
                .snap_route(&input.geometry, mode)
                .map_err(err_to_js)?,
        );*/
    }

    Ok(())
}

#[derive(Deserialize)]
struct GeoJsonLineString {
    #[serde(deserialize_with = "deserialize_geometry")]
    geometry: LineString,
}
