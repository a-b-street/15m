use std::fs::File;
use std::io::BufWriter;

use anyhow::Result;

use backend::{Graph, Timer};

/// This is a CLI tool to build a Graph file with GTFS data, for later use in the web app. This
/// gives a perf benefit (faster to load a pre-built graph) and allows GTFS use (no practical way
/// to read a huge GTFS file or clip it from web). The downside is having to manually manage these
/// prebuilt files as the format changes -- which is why, unlike in A/B Street, this'll just be for
/// manual testing for now.
fn main() -> Result<()> {
    simple_logger::SimpleLogger::new().init().unwrap();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || (args[1] != "graph" && args[1] != "fgb") {
        println!("Usage: one of these:");
        println!("To make a graph.bin: graph osm.pbf [gtfs_directory]");
        println!("To make a gtfs.fgb: fgb gtfs_directory");
        std::process::exit(1);
    }

    if args[1] == "graph" {
        let timer = Timer::new("build graph", None);
        let osm_bytes = std::fs::read(&args[2])?;

        let graph = Graph::new(&osm_bytes, args.get(3), timer)?;
        let writer = BufWriter::new(File::create("graph.bin")?);
        bincode::serialize_into(writer, &graph)?;
    } else if args[1] == "fgb" {
        let mut timer = Timer::new("build fgb from gtfs", None);
        timer.step("parse GTFS");
        let model = backend::GtfsModel::parse(&args[2], None)?;
        timer.step("turn into FGB");
        model.to_fgb("gtfs.fgb")?;
        timer.done();
    }

    Ok(())
}
