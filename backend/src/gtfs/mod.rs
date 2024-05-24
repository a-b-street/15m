use std::time::Duration;

use chrono::NaiveTime;
use geo::Point;
use geojson::{Feature, Geometry};
use serde::{Deserialize, Serialize};
use utils::Mercator;

use self::ids::orig_ids;
pub use self::ids::{StopID, TripID};
use crate::graph::RoadID;

mod ids;
mod scrape;

// TODO days of the week, exceptions, etc. a daily model for now.

#[derive(Serialize, Deserialize)]
pub struct GtfsModel {
    // Indexed by StopID and TripID
    pub stops: Vec<Stop>,
    pub trips: Vec<Trip>,
}

#[derive(Serialize, Deserialize)]
pub struct Stop {
    pub name: String,
    pub orig_id: orig_ids::StopID,
    pub point: Point,
    pub road: RoadID,
    // Sorted by time1
    pub next_steps: Vec<NextStep>,
}

/// `trip` arrives at some `Stop` at `time`. Then it reaches `stop2` at `time2`
#[derive(Serialize, Deserialize)]
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

impl GtfsModel {
    pub fn empty() -> Self {
        Self {
            stops: Vec::new(),
            trips: Vec::new(),
        }
    }

    /// Starting from a stop at some time, find all the next trips going somewhere, waiting up to
    /// max_wait.
    pub fn trips_from(&self, stop1: StopID, time: NaiveTime, max_wait: Duration) -> Vec<&NextStep> {
        // TODO Binary search
        let mut results = Vec::new();
        for next_step in &self.stops[stop1.0].next_steps {
            // These are sorted by time, so give up after we've seen enough
            if next_step.time1 > time + max_wait {
                break;
            }

            if next_step.time1 > time {
                results.push(next_step);
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
