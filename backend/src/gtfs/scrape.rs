use std::collections::BTreeMap;
use std::fs::File;

use anyhow::Result;
use chrono::NaiveTime;
use geo::Coord;
use serde::Deserialize;

use super::{GtfsModel, Stop, StopID, Trip, TripID};

/// Takes a path to a GTFS directory
pub fn parse(dir_path: &str) -> Result<GtfsModel> {
    println!("Scraping stops.txt");
    let mut stops: BTreeMap<StopID, Stop> = BTreeMap::new();
    for rec in csv::Reader::from_reader(File::open(format!("{dir_path}/stops.txt"))?).deserialize()
    {
        let rec: StopRow = rec?;
        stops.insert(
            rec.stop_id,
            Stop {
                name: rec.stop_name,
                point: Coord { x: rec.stop_lon, y: rec.stop_lat },
                arrivals: Vec::new(),
            },
        );
    }

    let mut trips: BTreeMap<TripID, Trip> = BTreeMap::new();
    println!("Scraping stop_times.txt");
    for rec in
        csv::Reader::from_reader(File::open(format!("{dir_path}/stop_times.txt"))?).deserialize()
    {
        let rec: StopTimeRow = rec?;
        let Ok(arrival_time) = NaiveTime::parse_from_str(&rec.arrival_time, "%H:%M:%S") else {
            // TODO Handle times > 24 hours
            continue;
        };

        // Which days does this stop occur on?
        let mut stop = stops.get_mut(&rec.stop_id).unwrap();
        stop.arrivals.push((rec.trip_id.clone(), arrival_time));

        trips
            .entry(rec.trip_id)
            .or_insert_with(|| Trip { stops: Vec::new() })
            .stops
            .push((rec.stop_id, arrival_time));
    }

    Ok(GtfsModel { stops, trips })
}

#[derive(Deserialize)]
struct StopRow {
    stop_id: StopID,
    stop_name: String,
    stop_lon: f64,
    stop_lat: f64,
}

#[derive(Deserialize)]
struct StopTimeRow {
    trip_id: TripID,
    stop_id: StopID,
    arrival_time: String,
}
