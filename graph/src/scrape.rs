use anyhow::Result;
use enum_map::EnumMap;
use geo::EuclideanLength;
use muv_osm::{AccessLevel, TMode};
use rstar::RTree;
use utils::Tags;

use crate::gtfs::{GtfsModel, StopID};
use crate::route::Router;
use crate::{
    Direction, EdgeLocation, Graph, GtfsSource, Intersection, IntersectionID, Mode, Road, RoadID,
    Timer,
};

impl Graph {
    /// Constructs a graph from OpenStreetMap data.
    ///
    /// - `input_bytes`: Bytes of an osm.pbf or osm.xml file
    /// - `osm_reader`: A callback for every OSM element read, to extract non-graph data
    /// - `modify_roads`: Runs before any routing structures are calculated. Use to modify access per mode.
    pub fn new<F: FnOnce(&mut Vec<Road>), R: utils::osm2graph::OsmReader>(
        input_bytes: &[u8],
        osm_reader: &mut R,
        modify_roads: F,
        timer: &mut Timer,
    ) -> Result<Graph> {
        timer.step("parse OSM and split graph");

        let graph = utils::osm2graph::Graph::new(
            input_bytes,
            // Don't do any filtering by Mode yet
            |tags| {
                tags.has("highway") && !tags.is("highway", "proposed") && !tags.is("area", "yes")
            },
            osm_reader,
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
        let mut roads: Vec<Road> = graph
            .edges
            .into_iter()
            .map(|e| {
                let access = calculate_access(&e.osm_tags);
                let mut max_speed = calculate_max_speed(&e.osm_tags);
                if max_speed == 0.0 {
                    error!(
                        "Zero maxspeed for {} ({:?}), boosting to 1mph",
                        e.osm_way, e.osm_tags
                    );
                    max_speed = 1.0 * 0.44704;
                }

                Road {
                    id: RoadID(e.id.0),
                    src_i: IntersectionID(e.src.0),
                    dst_i: IntersectionID(e.dst.0),
                    way: e.osm_way,
                    node1: e.osm_node1,
                    node2: e.osm_node2,
                    osm_tags: e.osm_tags,
                    length_meters: e.linestring.euclidean_length(),
                    linestring: e.linestring,

                    access,
                    max_speed,
                    stops: Vec::new(),
                }
            })
            .collect();

        modify_roads(&mut roads);

        timer.push("build closest_road");
        let closest_road = EnumMap::from_fn(|mode| {
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

        timer.push("building router");
        let router = EnumMap::from_fn(|mode| {
            timer.step(format!("for {mode:?}"));
            Router::new(&roads, mode)
        });
        timer.pop();

        Ok(Graph {
            roads,
            intersections,
            mercator: graph.mercator,
            closest_road,
            router,
            boundary_polygon: graph.boundary_polygon,

            gtfs: GtfsModel::empty(),
        })
    }

    /// Adds in GTFS data to the current graph. This only makes sense to call once.
    pub async fn setup_gtfs(&mut self, source: GtfsSource, timer: &mut Timer) -> Result<()> {
        timer.push("setting up GTFS");
        timer.step("parse");
        let mut gtfs = match source {
            GtfsSource::Dir(path) => GtfsModel::parse(&path, Some(&self.mercator))?,
            GtfsSource::Geomedea(url) => GtfsModel::from_geomedea(&url, &self.mercator).await?,
            GtfsSource::None => GtfsModel::empty(),
        };
        snap_stops(
            &mut self.roads,
            &mut gtfs,
            &self.closest_road[Mode::Foot],
            timer,
        );
        timer.pop();
        Ok(())
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

fn snap_stops(
    roads: &mut Vec<Road>,
    gtfs: &mut GtfsModel,
    closest_road: &RTree<EdgeLocation>,
    timer: &mut Timer,
) {
    if gtfs.stops.is_empty() {
        return;
    }

    timer.step("find closest roads per stop");
    // TODO Make an iterator method that returns the IDs too
    for (idx, stop) in gtfs.stops.iter_mut().enumerate() {
        let stop_id = StopID(idx);
        if let Some(r) = closest_road.nearest_neighbor(&stop.point.into()) {
            // TODO Limit how far away we snap, or use the boundary polygon
            roads[r.data.0].stops.push(stop_id);
            stop.road = r.data;
        } else {
            // TODO Need to get rid of the stop
            error!("{stop_id:?} didn't snap to any road");
        }
    }
}
