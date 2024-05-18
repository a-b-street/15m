use std::collections::BTreeMap;

use chrono::NaiveTime;
use geo::Coord;
use serde::{Deserialize, Serialize};

mod scrape;

// TODO cheap numeric IDs, later
// TODO days of the week, exceptions, etc. a daily model for now.

#[derive(Serialize, Deserialize)]
pub struct GtfsModel {
    pub stops: BTreeMap<StopID, Stop>,
    pub trips: BTreeMap<TripID, Trip>,
}

#[derive(Serialize, Deserialize)]
pub struct Stop {
    pub name: String,
    pub point: Coord,
    pub arrivals: Vec<(TripID, NaiveTime)>,
    // Or maybe even... (arrival time, tripid, next stop ID and arrival time there)
}

#[derive(Serialize, Deserialize)]
pub struct Trip {
    // with arrival time
    pub stops: Vec<(StopID, NaiveTime)>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StopID(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TripID(String);

impl GtfsModel {
    pub fn empty() -> Self {
        Self {
            stops: BTreeMap::new(),
            trips: BTreeMap::new(),
        }
    }
}
