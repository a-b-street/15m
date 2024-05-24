use std::collections::BTreeMap;
use std::fs::File;

use anyhow::Result;
use chrono::NaiveTime;
use geo::{Contains, Point};
use serde::Deserialize;
use utils::Mercator;

use super::ids::{orig_ids, IDMapping};
use super::{GtfsModel, NextStep, Stop, StopID, Trip, TripID};
use crate::graph::RoadID;

impl GtfsModel {
    /// Takes a path to a GTFS directory
    pub fn parse(dir_path: &str, mercator: &Mercator) -> Result<GtfsModel> {
        println!("Scraping stops.txt");
        let mut stop_ids: IDMapping<orig_ids::StopID, StopID> = IDMapping::new();
        let mut stops: Vec<Stop> = Vec::new();
        for rec in
            csv::Reader::from_reader(File::open(format!("{dir_path}/stops.txt"))?).deserialize()
        {
            let rec: StopRow = rec?;

            // TODO Move code to utils
            let point = Point::new(rec.stop_lon, rec.stop_lat);
            if !mercator.wgs84_bounds.contains(&point) {
                continue;
            }

            stop_ids.insert_new(rec.stop_id.clone())?;
            stops.push(Stop {
                name: rec.stop_name,
                orig_id: rec.stop_id,
                point: mercator.to_mercator(&point),
                next_steps: Vec::new(),
                // Dummy value, fill out later
                road: RoadID(0),
            });
        }

        println!("Scraping stop_times.txt");
        let mut trips_table: BTreeMap<orig_ids::TripID, Trip> = BTreeMap::new();
        for rec in csv::Reader::from_reader(File::open(format!("{dir_path}/stop_times.txt"))?)
            .deserialize()
        {
            let rec: StopTimeRow = rec?;
            let Ok(arrival_time) = NaiveTime::parse_from_str(&rec.arrival_time, "%H:%M:%S") else {
                // TODO Handle times > 24 hours
                continue;
            };

            // Skip out-of-bounds stops
            let Some(stop_id) = stop_ids.get(&rec.stop_id) else {
                continue;
            };

            trips_table
                .entry(rec.trip_id)
                .or_insert_with(|| Trip {
                    stop_sequence: Vec::new(),
                })
                .stop_sequence
                .push((stop_id, arrival_time));
        }

        // Produce a compact Trips vec
        let trips: Vec<Trip> = trips_table.into_values().collect();

        // Precompute the next steps from each stop
        for (idx, trip) in trips.iter().enumerate() {
            let trip_id = TripID(idx);
            for pair in trip.stop_sequence.windows(2) {
                let (stop1, time1) = &pair[0];
                let (stop2, time2) = &pair[1];
                stops[stop1.0].next_steps.push(NextStep {
                    time1: *time1,
                    trip: trip_id,
                    stop2: *stop2,
                    time2: *time2,
                });
            }
        }

        for stop in &mut stops {
            stop.next_steps.sort_by_key(|x| x.time1);
        }

        Ok(GtfsModel { stops, trips })
    }
}

#[derive(Deserialize)]
struct StopRow {
    stop_id: orig_ids::StopID,
    stop_name: String,
    stop_lon: f64,
    stop_lat: f64,
}

#[derive(Deserialize)]
struct StopTimeRow {
    trip_id: orig_ids::TripID,
    stop_id: orig_ids::StopID,
    arrival_time: String,
}
