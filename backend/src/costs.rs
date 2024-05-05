use std::time::Duration;

use geo::EuclideanLength;

use crate::graph::{Mode, Road};

pub fn cost(road: &Road, mode: Mode) -> Duration {
    // TODO Configurable
    // 10 mph
    let max_bicycle_speed = 4.4704;
    // 3 mph
    let max_foot_speed = 1.34112;

    // All speeds are meters/second, so the units work out
    let distance = road.linestring.euclidean_length();
    match mode {
        Mode::Car => Duration::from_secs_f64(distance / road.max_speed),
        // TODO Use elevation and other more detailed things
        Mode::Bicycle => Duration::from_secs_f64(distance / max_bicycle_speed),
        Mode::Foot => Duration::from_secs_f64(distance / max_foot_speed),
    }
}
