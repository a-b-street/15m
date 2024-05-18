use std::collections::HashMap;

use anyhow::Result;
use enum_map::EnumMap;
use geo::{Coord, EuclideanLength, LineString};
use muv_osm::{AccessLevel, TMode};
use osm_reader::OsmID;
use rstar::primitives::GeomWithData;
use rstar::RTree;
use utils::Tags;

use crate::amenity::Amenity;
use crate::graph::{
    AmenityID, Direction, Graph, Intersection, IntersectionID, IntersectionLocation, Mode, Road,
    RoadID,
};
use crate::gtfs::GtfsModel;
use crate::route::Router;
use crate::timer::Timer;

struct ReadAmenities {
    amenities: Vec<Amenity>,
}

impl utils::osm2graph::OsmReader for ReadAmenities {
    fn node(&mut self, id: osm_reader::NodeID, pt: Coord, tags: Tags) {
        self.amenities.extend(Amenity::maybe_new(
            &tags,
            OsmID::Node(id),
            pt.into(),
            AmenityID(self.amenities.len()),
        ));
    }

    fn way(
        &mut self,
        id: osm_reader::WayID,
        node_ids: &Vec<osm_reader::NodeID>,
        node_mapping: &HashMap<osm_reader::NodeID, Coord>,
        tags: &Tags,
    ) {
        self.amenities.extend(Amenity::maybe_new(
            tags,
            OsmID::Way(id),
            // TODO Centroid
            node_mapping[&node_ids[0]].into(),
            AmenityID(self.amenities.len()),
        ));
    }

    // TODO Are there amenities as relations?
}

impl Graph {
    /// Call with bytes of an osm.pbf or osm.xml string
    pub fn new(input_bytes: &[u8], gtfs_dir: Option<&String>, mut timer: Timer) -> Result<Graph> {
        timer.step("parse OSM and split graph");

        let mut amenities = ReadAmenities {
            amenities: Vec::new(),
        };
        let graph = utils::osm2graph::Graph::new(
            input_bytes,
            |tags| {
                tags.has("highway") && !tags.is("highway", "proposed") && !tags.is("area", "yes")
            },
            &mut amenities,
        )?;

        timer.step("calculate road attributes");
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
                let max_speed = calculate_max_speed(&e.osm_tags);
                Road {
                    id: RoadID(e.id.0),
                    src_i: IntersectionID(e.src.0),
                    dst_i: IntersectionID(e.dst.0),
                    way: e.osm_way,
                    node1: e.osm_node1,
                    node2: e.osm_node2,
                    length_meters: e.linestring.euclidean_length(),
                    linestring: e.linestring,

                    access,
                    max_speed,
                    tags: e.osm_tags,
                    amenities: EnumMap::default(),
                    stops: Vec::new(),
                }
            })
            .collect();
        for a in &mut amenities.amenities {
            a.point = graph.mercator.pt_to_mercator(a.point.into()).into();
        }

        snap_amenities(&mut roads, &amenities.amenities, &mut timer);

        timer.push("build closest_intersection");
        let closest_intersection = EnumMap::from_fn(|mode| {
            timer.step(format!("for {mode:?}"));
            let mut points = Vec::new();
            for i in &intersections {
                if i.roads
                    .iter()
                    .any(|r| roads[r.0].allows_forwards(mode) || roads[r.0].allows_backwards(mode))
                {
                    points.push(IntersectionLocation::new(i.point.into(), i.id));
                }
            }
            RTree::bulk_load(points)
        });
        timer.pop();

        timer.push("building router");
        let router = EnumMap::from_fn(|mode| {
            timer.step(format!("for {mode:?}"));
            Router::new(&roads, mode)
        });
        timer.pop();

        timer.push("setting up GTFS");
        timer.step("parse");
        let gtfs = if let Some(path) = gtfs_dir {
            GtfsModel::parse(path, &graph.mercator)?
        } else {
            GtfsModel::empty()
        };
        snap_stops(&mut roads, &gtfs, &mut timer);
        timer.pop();

        timer.done();

        Ok(Graph {
            roads,
            intersections,
            mercator: graph.mercator,
            closest_intersection,
            router,
            boundary_polygon: graph.boundary_polygon,

            amenities: amenities.amenities,
            gtfs,
        })
    }
}

// TODO Should also look at any barriers
fn calculate_access(tags: &Tags) -> EnumMap<Mode, Direction> {
    let tags: muv_osm::Tag = tags.0.iter().collect();
    let regions: [&'static str; 0] = [];
    let lanes = muv_osm::lanes::highway_lanes(&tags, &regions).unwrap();

    let mut forwards: EnumMap<Mode, bool> = EnumMap::default();
    let mut backwards: EnumMap<Mode, bool> = EnumMap::default();

    // TODO Check if this logic is correct
    for lane in lanes.lanes {
        if let muv_osm::lanes::LaneVariant::Travel(lane) = lane.variant {
            for (direction_per_mode, lane_direction) in [
                (&mut forwards, &lane.forward),
                (&mut backwards, &lane.backward),
            ] {
                for (mode, muv_mode) in [
                    (Mode::Car, TMode::Motorcar),
                    (Mode::Bicycle, TMode::Bicycle),
                    (Mode::Foot, TMode::Foot),
                ] {
                    if let Some(conditional_access) = lane_direction.access.get(muv_mode) {
                        if let Some(access) = conditional_access.base() {
                            if access_level_allowed(access) {
                                direction_per_mode[mode] = true;
                            }
                        }
                    }

                    if let Some(conditional_speed) = lane_direction.maxspeed.get(muv_mode) {
                        if let Some(_speed) = conditional_speed.base() {
                            // TODO
                        }
                    }
                }
            }
        }
    }

    EnumMap::from_fn(|mode| bool_to_dir(forwards[mode], backwards[mode]))
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
            return 0.277778 * kmph;
        }
        if let Some(mph) = x.strip_suffix(" mph").and_then(|x| x.parse::<f64>().ok()) {
            return 0.44704 * mph;
        }
    }
    // Arbitrary fallback
    30.0 * 0.44704
}

type EdgeLocation = GeomWithData<LineString, RoadID>;

fn snap_amenities(roads: &mut Vec<Road>, amenities: &Vec<Amenity>, timer: &mut Timer) {
    timer.push("snap amenities");
    timer.push("build closest_per_mode");
    let closest_per_mode = EnumMap::from_fn(|mode| {
        timer.step(format!("for {mode:?}"));
        RTree::bulk_load(
            roads
                .iter()
                .filter(|r| r.access[mode] != Direction::None)
                .map(|r| EdgeLocation::new(r.linestring.clone(), r.id))
                .collect(),
        )
    });
    timer.pop();
    timer.step("find closest roads");
    for amenity in amenities {
        for (mode, closest) in &closest_per_mode {
            if let Some(r) = closest.nearest_neighbor(&amenity.point) {
                roads[r.data.0].amenities[mode].push(amenity.id);
            }
        }
    }
    timer.pop();
}

fn snap_stops(roads: &mut Vec<Road>, gtfs: &GtfsModel, timer: &mut Timer) {
    if gtfs.stops.is_empty() {
        return;
    }

    // Only care about one mode
    // TODO Could we reuse from snap_amenities for some perf?
    timer.step("build closest_road");
    let closest_road = RTree::bulk_load(
        roads
            .iter()
            .filter(|r| r.access[Mode::Foot] != Direction::None)
            .map(|r| EdgeLocation::new(r.linestring.clone(), r.id))
            .collect(),
    );

    timer.step("find closest roads per stop");
    for (stop_id, stop) in &gtfs.stops {
        if let Some(r) = closest_road.nearest_neighbor(&stop.point.into()) {
            // TODO Limit how far away we snap, or use the boundary polygon
            roads[r.data.0].stops.push(stop_id.clone());
        }
    }
}
