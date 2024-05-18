use std::collections::BTreeMap;

use chrono::NaiveTime;
use geo::Coord;
use serde::Deserialize;

mod scrape;

// TODO cheap numeric IDs, later
// TODO days of the week, exceptions, etc. a daily model for now.

pub struct GtfsModel {
    pub stops: BTreeMap<StopID, Stop>,
    pub trips: BTreeMap<TripID, Trip>,
}

pub struct Stop {
    pub name: String,
    pub point: Coord,
    pub arrivals: Vec<(TripID, NaiveTime)>,
    // Or maybe even... (arrival time, tripid, next stop ID and arrival time there)
}

pub struct Trip {
    // with arrival time
    pub stops: Vec<(StopID, NaiveTime)>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct StopID(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct TripID(String);
