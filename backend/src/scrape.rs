use anyhow::Result;
use muv_osm::{AccessLevel, TMode};
use rstar::RTree;
use utils::Tags;

use crate::graph::{
    Direction, Graph, Intersection, IntersectionID, IntersectionLocation, Road, RoadID,
};

pub fn scrape_osm(input_bytes: &[u8]) -> Result<Graph> {
    let graph = utils::osm2graph::Graph::new(input_bytes, |tags| {
        tags.has("highway") && !tags.is("highway", "proposed") && !tags.is("area", "yes")
    })?;

    // Copy all the fields
    let intersections: Vec<Intersection> = graph
        .intersections
        .into_iter()
        .map(|i| Intersection {
            id: IntersectionID(i.id.0),
            point: i.point,
            node: i.osm_node,
            roads: i.edges.into_iter().map(|e| RoadID(e.0)).collect(),
        })
        .collect();

    // Add in a bit
    let roads = graph
        .edges
        .into_iter()
        .map(|e| {
            let (access_car, access_bicycle, access_foot) = calculate_access(&e.osm_tags);
            Road {
                id: RoadID(e.id.0),
                src_i: IntersectionID(e.src.0),
                dst_i: IntersectionID(e.dst.0),
                way: e.osm_way,
                node1: e.osm_node1,
                node2: e.osm_node2,
                linestring: e.linestring,

                access_car,
                access_bicycle,
                access_foot,
                tags: e.osm_tags,
            }
        })
        .collect();

    let mut points = Vec::new();
    for i in &intersections {
        points.push(IntersectionLocation::new(i.point.into(), i.id));
    }
    let closest_intersection = RTree::bulk_load(points);

    Ok(Graph {
        roads,
        intersections,
        mercator: graph.mercator,
        closest_intersection,
        boundary_polygon: graph.boundary_polygon,
    })
}

// TODO Should also look at any barriers
fn calculate_access(tags: &Tags) -> (Direction, Direction, Direction) {
    let tags: muv_osm::Tag = tags.0.iter().collect();
    let regions: [&'static str; 0] = [];
    let lanes = muv_osm::lanes::highway_lanes(&tags, &regions).unwrap();

    let mut car_forwards = false;
    let mut car_backwards = false;
    let mut bicycle_forwards = false;
    let mut bicycle_backwards = false;
    // TODO Is one-way ever possible?
    let mut foot_forwards = false;
    let mut foot_backwards = false;

    // TODO Check if this logic is correct
    for lane in lanes.lanes {
        if let muv_osm::lanes::LaneVariant::Travel(lane) = lane.variant {
            for (bit, mode) in [
                (&mut car_forwards, TMode::Motorcar),
                (&mut bicycle_forwards, TMode::Bicycle),
                (&mut foot_forwards, TMode::Foot),
            ] {
                if let Some(conditional_access) = lane.forward.access.get(mode) {
                    if let Some(access) = conditional_access.base() {
                        if access_level_allowed(access) {
                            *bit = true;
                        }
                    }
                }
            }

            for (bit, mode) in [
                (&mut car_backwards, TMode::Motorcar),
                (&mut bicycle_backwards, TMode::Bicycle),
                (&mut foot_backwards, TMode::Foot),
            ] {
                if let Some(conditional_access) = lane.backward.access.get(mode) {
                    if let Some(access) = conditional_access.base() {
                        if access_level_allowed(access) {
                            *bit = true;
                        }
                    }
                }
            }
        }
    }

    (
        bool_to_dir(car_forwards, car_backwards),
        bool_to_dir(bicycle_forwards, bicycle_backwards),
        bool_to_dir(foot_forwards, foot_backwards),
    )
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
