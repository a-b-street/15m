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

// Move to mod after deciding to store every day
#[derive(Clone, Copy, Debug, PartialEq)]
enum Day {
    Monday = 0,
    Tuesday = 1,
    Wednesday = 2,
    Thursday = 3,
    Friday = 4,
    Saturday = 5,
    Sunday = 6,
}

impl GtfsModel {
    /// Takes a path to a GTFS directory
    pub fn parse(dir_path: &str, mercator: &Mercator) -> Result<GtfsModel> {
        info!("Scraping trips.txt");
        let mut trip_to_service: BTreeMap<orig_ids::TripID, orig_ids::ServiceID> = BTreeMap::new();
        for rec in
            csv::Reader::from_reader(File::open(format!("{dir_path}/trips.txt"))?).deserialize()
        {
            let rec: TripRow = rec?;
            trip_to_service.insert(rec.trip_id, rec.service_id);
        }

        info!("Scraping calendar.txt");
        let mut service_to_days: BTreeMap<orig_ids::ServiceID, Vec<Day>> = BTreeMap::new();
        for rec in
            csv::Reader::from_reader(File::open(format!("{dir_path}/calendar.txt"))?).deserialize()
        {
            let rec: CalendarRow = rec?;
            let mut days = Vec::new();
            for (day, include) in [
                (Day::Monday, rec.monday),
                (Day::Tuesday, rec.tuesday),
                (Day::Wednesday, rec.wednesday),
                (Day::Thursday, rec.thursday),
                (Day::Friday, rec.friday),
                (Day::Saturday, rec.saturday),
                (Day::Sunday, rec.sunday),
            ] {
                if include == 1 {
                    days.push(day);
                }
            }
            service_to_days.insert(rec.service_id, days);
        }

        info!("Scraping stops.txt");
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

        info!("Scraping stop_times.txt");
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

            // Which days does this stop occur on?
            let service = &trip_to_service[&rec.trip_id];
            let Some(days) = service_to_days.get(service) else {
                warn!("Don't know what days service {service:?} is on");
                continue;
            };

            // TODO For now, only keep Monday
            if !days.contains(&Day::Monday) {
                continue;
            }

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
struct TripRow {
    trip_id: orig_ids::TripID,
    service_id: orig_ids::ServiceID,
}

#[derive(Deserialize)]
struct CalendarRow {
    service_id: orig_ids::ServiceID,
    monday: usize,
    tuesday: usize,
    wednesday: usize,
    thursday: usize,
    friday: usize,
    saturday: usize,
    sunday: usize,
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
