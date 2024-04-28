use anyhow::Result;
use rstar::RTree;
use utils::Tags;

use crate::{Direction, Intersection, IntersectionID, IntersectionLocation, MapModel, Road, RoadID};

pub fn scrape_osm(input_bytes: &[u8]) -> Result<MapModel> {
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
        .map(|e| Road {
            id: RoadID(e.id.0),
            src_i: IntersectionID(e.src.0),
            dst_i: IntersectionID(e.dst.0),
            way: e.osm_way,
            node1: e.osm_node1,
            node2: e.osm_node2,
            linestring: e.linestring,

            access_car: is_car_allowed(&e.osm_tags),
            access_bicycle: is_bicycle_allowed(&e.osm_tags),
            access_foot: is_foot_allowed(&e.osm_tags),
            tags: e.osm_tags,
        })
        .collect();

    let mut points = Vec::new();
    for i in &intersections {
        points.push(IntersectionLocation::new(i.point.into(), i.id));
    }
    let closest_intersection = RTree::bulk_load(points);

    Ok(MapModel {
        roads,
        intersections,
        mercator: graph.mercator,
        closest_intersection,
        boundary_polygon: graph.boundary_polygon,
    })
}

// TODO Use Muv (rstar is pinning to an old smallvec). This is just placeholder.
fn is_car_allowed(tags: &Tags) -> Direction {
    if tags.is_any("highway", vec![
        "footway",
        "steps",
        "path",
        "track",
        "corridor",
        "crossing",
        "pedestrian",
    ]) {
        return Direction::None;
    }
    Direction::Both
}
fn is_bicycle_allowed(_tags: &Tags) -> Direction {
    Direction::Both
}
fn is_foot_allowed(tags: &Tags) -> Direction {
    if tags.is_any("highway", vec!["motorway", "motorway_link"]) {
        return Direction::None;
    }
    Direction::Both
}
