use std::fs::File;
use std::io::BufWriter;

use anyhow::Result;

use backend::MapModel;
use graph::{GtfsModel, Timer};

// TODO Don't need tokio multithreading, but fighting config to get single working
/// This is a CLI tool to build a MapModel file, for later use in the web app or CLI. This gives a
/// perf benefit (faster to load a pre-built graph), but manually managing these prebuilt files as
/// the format changes is tedious. That's why, unlike in A/B Street, this'll just be for manual
/// testing for now.
#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 || (args[1] != "graph" && args[1] != "gmd") {
        println!("Usage: one of these:");
        println!("To make a graph.bin: graph osm.pbf");
        println!("To make a gtfs.gmd: gmd gtfs_directory");
        std::process::exit(1);
    }

    if args[1] == "gmd" {
        let mut timer = Timer::new("build geomedea from gtfs", None);
        timer.step("parse GTFS");
        let model = GtfsModel::parse(&args[2], None)?;
        timer.step("turn into geomedea");
        model.to_geomedea("gtfs.gmd")?;
        timer.done();
        return Ok(());
    }

    let mut timer = Timer::new("build model", None);
    let osm_bytes = std::fs::read(&args[2])?;
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
