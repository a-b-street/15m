use std::collections::BTreeMap;
use std::fs::File;

use anyhow::Result;
use chrono::NaiveTime;
use geo::{Contains, Point};
use serde::Deserialize;
use utils::Mercator;

use super::{GtfsModel, NextStep, Stop, StopID, Trip, TripID};
use crate::graph::RoadID;

impl GtfsModel {
    /// Takes a path to a GTFS directory
    pub fn parse(dir_path: &str, mercator: &Mercator) -> Result<GtfsModel> {
        println!("Scraping stops.txt");
        let mut stops: BTreeMap<StopID, Stop> = BTreeMap::new();
        for rec in
            csv::Reader::from_reader(File::open(format!("{dir_path}/stops.txt"))?).deserialize()
        {
            let rec: StopRow = rec?;

            // TODO Move code to utils
            let point = Point::new(rec.stop_lon, rec.stop_lat);
            if !mercator.wgs84_bounds.contains(&point) {
                continue;
            }

            stops.insert(
                rec.stop_id,
                Stop {
                    name: rec.stop_name,
                    point: mercator.to_mercator(&point),
                    next_steps: Vec::new(),
                    // Dummy value, fill out later
                    road: RoadID(0),
                },
            );
        }

        let mut trips: BTreeMap<TripID, Trip> = BTreeMap::new();
        println!("Scraping stop_times.txt");
        for rec in csv::Reader::from_reader(File::open(format!("{dir_path}/stop_times.txt"))?)
            .deserialize()
        {
            let rec: StopTimeRow = rec?;
            let Ok(arrival_time) = NaiveTime::parse_from_str(&rec.arrival_time, "%H:%M:%S") else {
                // TODO Handle times > 24 hours
                continue;
            };

            // Skip out-of-bounds stops
            if !stops.contains_key(&rec.stop_id) {
                continue;
            }

            trips
                .entry(rec.trip_id)
                .or_insert_with(|| Trip {
                    stop_sequence: Vec::new(),
                })
                .stop_sequence
                .push((rec.stop_id, arrival_time));
        }

        // Precompute the next steps from each stop
        for (trip_id, trip) in &trips {
            for pair in trip.stop_sequence.windows(2) {
                let (stop1, time1) = &pair[0];
                let (stop2, time2) = &pair[1];
                stops.get_mut(&stop1).unwrap().next_steps.push(NextStep {
                    time1: *time1,
                    trip: trip_id.clone(),
                    stop2: stop2.clone(),
                    time2: *time2,
                });
            }
        }

        for stop in stops.values_mut() {
            stop.next_steps.sort_by_key(|x| x.time1);
        }

        Ok(GtfsModel { stops, trips })
    }
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
