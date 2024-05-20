use std::collections::BTreeMap;
use std::time::Duration;

use chrono::NaiveTime;
use geo::Point;
use geojson::{Feature, Geometry};
use serde::{Deserialize, Serialize};
use utils::Mercator;

use crate::graph::RoadID;

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
    pub point: Point,
    pub road: RoadID,
    // Sorted by time1
    pub next_steps: Vec<NextStep>,
}

/// `trip` arrives at some `Stop` at `time`. Then it reaches `stop2` at `time2`
#[derive(Clone, Serialize, Deserialize)]
pub struct NextStep {
    pub time1: NaiveTime,
    pub trip: TripID,
    pub stop2: StopID,
    pub time2: NaiveTime,
}

#[derive(Serialize, Deserialize)]
pub struct Trip {
    // (stop, arrival time) in order
    pub stop_sequence: Vec<(StopID, NaiveTime)>,
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

    /// Starting from a stop at some time, find all the next trips going somewhere, waiting up to
    /// max_wait.
    pub fn trips_from(&self, stop1: &StopID, time: NaiveTime, max_wait: Duration) -> Vec<NextStep> {
        // TODO Improve with compact IDs, binary search, etc
        let mut results = Vec::new();
        for next_step in &self.stops[stop1].next_steps {
            // These are sorted by time, so give up after we've seen enough
            if next_step.time1 > time + max_wait {
                break;
            }

            if next_step.time1 > time {
                results.push(next_step.clone());
            }
        }
        results
    }
}

impl Stop {
    pub fn to_gj(&self, mercator: &Mercator) -> Feature {
        let mut f = Feature::from(Geometry::from(&mercator.to_wgs84(&self.point)));
        f.set_property("name", self.name.clone());
        f.set_property(
            "next_steps",
            serde_json::to_value(&self.next_steps).unwrap(),
        );
        f
    }
}
