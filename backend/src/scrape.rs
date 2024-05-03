use std::collections::HashMap;

use anyhow::Result;
use enum_map::{enum_map, EnumMap};
use geo::{Coord, LineString};
use muv_osm::{AccessLevel, TMode};
use osm_reader::{Element, OsmID};
use rstar::primitives::GeomWithData;
use rstar::RTree;
use utils::Tags;

use crate::amenity::Amenity;
use crate::graph::{
    AmenityID, Direction, Graph, Intersection, IntersectionID, IntersectionLocation, Mode, Road,
    RoadID,
};

pub fn scrape_osm(input_bytes: &[u8]) -> Result<Graph> {
    info!("Parsing {} bytes of OSM data", input_bytes.len());
    // This doesn't use osm2graph's helper, because it needs to scrape more things from OSM
    let mut node_mapping = HashMap::new();
    let mut highways = Vec::new();
    let mut amenities = Vec::new();
    osm_reader::parse(input_bytes, |elem| match elem {
        Element::Node {
            id, lon, lat, tags, ..
        } => {
            let pt = Coord { x: lon, y: lat };
            node_mapping.insert(id, pt);

            let tags = tags.into();
            amenities.extend(Amenity::maybe_new(
                &tags,
                OsmID::Node(id),
                pt.into(),
                AmenityID(amenities.len()),
            ));
        }
        Element::Way {
            id,
            mut node_ids,
            tags,
            ..
        } => {
            let tags: Tags = tags.into();

            amenities.extend(Amenity::maybe_new(
                &tags,
                OsmID::Way(id),
                // TODO Centroid
                node_mapping[&node_ids[0]].into(),
                AmenityID(amenities.len()),
            ));

            if tags.has("highway") && !tags.is("highway", "proposed") && !tags.is("area", "yes") {
                // TODO This sometimes happens from Overpass?
                let num = node_ids.len();
                node_ids.retain(|n| node_mapping.contains_key(n));
                if node_ids.len() != num {
                    warn!("{id} refers to nodes outside the imported area");
                }
                if node_ids.len() >= 2 {
                    highways.push(utils::osm2graph::Way { id, node_ids, tags });
                }
            }
        }
        // TODO Amenity relations?
        Element::Relation { .. } => {}
        Element::Bounds { .. } => {}
    })?;

    info!("Splitting {} ways into edges", highways.len());
    let graph = utils::osm2graph::Graph::from_scraped_osm(node_mapping, highways);

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
    let mut roads = graph
        .edges
        .into_iter()
        .map(|e| {
            let access = calculate_access(&e.osm_tags);
            Road {
                id: RoadID(e.id.0),
                src_i: IntersectionID(e.src.0),
                dst_i: IntersectionID(e.dst.0),
                way: e.osm_way,
                node1: e.osm_node1,
                node2: e.osm_node2,
                linestring: e.linestring,

                access,
                tags: e.osm_tags,
                amenities: EnumMap::default(),
            }
        })
        .collect();
    for a in &mut amenities {
        a.point = graph.mercator.pt_to_mercator(a.point.into()).into();
    }

    snap_amenities(&mut roads, &amenities);

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

        amenities,
    })
}

// TODO Should also look at any barriers
fn calculate_access(tags: &Tags) -> EnumMap<Mode, Direction> {
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

    enum_map! {
        Mode::Car => bool_to_dir(car_forwards, car_backwards),
        Mode::Bicycle => bool_to_dir(bicycle_forwards, bicycle_backwards),
        Mode::Foot => bool_to_dir(foot_forwards, foot_backwards),
    }
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

type EdgeLocation = GeomWithData<LineString, RoadID>;

fn snap_amenities(roads: &mut Vec<Road>, amenities: &Vec<Amenity>) {
    let closest_per_mode = EnumMap::from_fn(|mode| {
        RTree::bulk_load(
            roads
                .iter()
                .filter(|r| r.access[mode] != Direction::None)
                .map(|r| EdgeLocation::new(r.linestring.clone(), r.id))
                .collect(),
        )
    });
    for amenity in amenities {
        for (mode, closest) in &closest_per_mode {
            if let Some(r) = closest.nearest_neighbor(&amenity.point) {
                roads[r.data.0].amenities[mode].push(amenity.id);
            }
        }
    }
}
