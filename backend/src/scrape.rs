use anyhow::Result;
use rstar::RTree;

use crate::{Intersection, IntersectionID, IntersectionLocation, MapModel, Road, RoadID};

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
