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
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: osm.pbf [gtfs directory]");
        std::process::exit(1);
    }
    // TODO Enable a simple logger backend

    let timer = Timer::new("build graph", None);
    let osm_bytes = std::fs::read(&args[1])?;

    let graph = Graph::new(&osm_bytes, args.get(2), timer)?;
    let writer = BufWriter::new(File::create("graph.bin")?);
    bincode::serialize_into(writer, &graph)?;

    Ok(())
}
