use std::collections::BTreeMap;

use chrono::NaiveTime;
use geo::Point;
use geojson::{Feature, Geometry};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utils::Mercator;

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

impl Stop {
    pub fn to_gj(&self, mercator: &Mercator) -> Feature {
        let mut f = Feature::from(Geometry::from(&mercator.to_wgs84(&self.point)));
        f.set_property("name", self.name.clone());
        f.set_property(
            "arrivals",
            self.arrivals
                .iter()
                .map(|(trip_id, time)| json!([trip_id, time]))
                .collect::<Vec<_>>(),
        );
        f
    }
}
