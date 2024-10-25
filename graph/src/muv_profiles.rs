use std::time::Duration;

use geo::{EuclideanLength, LineString};
use muv_osm::{AccessLevel, TMode};
use utils::Tags;

use crate::Direction;

// TODO Separate profiles like this will repeat work parsing!

pub fn muv_car_profile() -> (
    String,
    Box<dyn Fn(&Tags, &LineString) -> (Direction, Duration)>,
) {
    (
        "car".to_string(),
        Box::new(|tags, linestring| {
            let access = calculate_access(tags, TMode::Motorcar);
            let cost =
                Duration::from_secs_f64(linestring.euclidean_length() / calculate_max_speed(tags));
            (access, cost)
        }),
    )
}

pub fn muv_bicycle_profile() -> (
    String,
    Box<dyn Fn(&Tags, &LineString) -> (Direction, Duration)>,
) {
    (
        "bicycle".to_string(),
        Box::new(|tags, linestring| {
            let access = calculate_access(tags, TMode::Bicycle);
            // TODO Use elevation and other more detailed things
            // 10 mph
            let max_bicycle_speed = 4.4704;
            let cost = Duration::from_secs_f64(linestring.euclidean_length() / max_bicycle_speed);
            (access, cost)
        }),
    )
}

pub fn muv_pedestrian_profile() -> (
    String,
    Box<dyn Fn(&Tags, &LineString) -> (Direction, Duration)>,
) {
    (
        "foot".to_string(),
        Box::new(|tags, linestring| {
            let access = calculate_access(tags, TMode::Foot);
            // TODO Use elevation and other more detailed things
            // 3 mph
            let max_foot_speed = 1.34112;
            let cost = Duration::from_secs_f64(linestring.euclidean_length() / max_foot_speed);
            (access, cost)
        }),
    )
}
// TODO Should also look at any barriers
fn calculate_access(tags: &Tags, mode: TMode) -> Direction {
    let tags: muv_osm::Tag = tags.0.iter().collect();
    let regions: [&'static str; 0] = [];
    let lanes = muv_osm::lanes::highway_lanes(&tags, &regions).unwrap();

    let mut forwards = false;
    let mut backwards = false;

    // TODO Check if this logic is correct
    for lane in lanes.lanes {
        if let muv_osm::lanes::LaneVariant::Travel(lane) = lane.variant {
            for (direction, lane_direction) in [
                (&mut forwards, &lane.forward),
                (&mut backwards, &lane.backward),
            ] {
                if let Some(conditional_access) = lane_direction.access.get(mode) {
                    if let Some(access) = conditional_access.base() {
                        if access_level_allowed(access) {
                            *direction = true;
                        }
                    }
                }

                if let Some(conditional_speed) = lane_direction.maxspeed.get(mode) {
                    if let Some(_speed) = conditional_speed.base() {
                        // TODO
                    }
                }
            }
        }
    }

    bool_to_dir(forwards, backwards)
}

fn access_level_allowed(access: &AccessLevel) -> bool {
    matches!(
        access,
        AccessLevel::Designated
            | AccessLevel::Yes
            | AccessLevel::Permissive
            | AccessLevel::Discouraged
            | AccessLevel::Destination
            | AccessLevel::Customers
            | AccessLevel::Private
    )
}

fn bool_to_dir(f: bool, b: bool) -> Direction {
    if f && b {
        Direction::Both
    } else if f {
        Direction::Forwards
    } else if b {
        Direction::Backwards
    } else {
        Direction::None
    }
}

fn calculate_max_speed(tags: &Tags) -> f64 {
    // TODO Use muv
    if let Some(x) = tags.get("maxspeed") {
        if let Some(kmph) = x.parse::<f64>().ok() {
            return 0.277778 * check_nonzero(kmph);
        }
        if let Some(mph) = x.strip_suffix(" mph").and_then(|x| x.parse::<f64>().ok()) {
            return 0.44704 * check_nonzero(mph);
        }
    }
    // Arbitrary fallback
    30.0 * 0.44704
}

fn check_nonzero(x: f64) -> f64 {
    if x == 0.0 {
        error!("Zero maxspeed, boosting to 1mph");
        1.0
    } else {
        x
    }
}
