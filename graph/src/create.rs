use std::collections::BTreeMap;
use std::time::Duration;

use anyhow::Result;
use geo::{Coord, Euclidean, Length, Line, LineString, Point};
use utils::Tags;

use crate::gtfs::GtfsModel;
use crate::route::Router;
use crate::{Direction, Graph, Intersection, IntersectionID, ProfileID, Road, RoadID, Timer};

impl Graph {
    /// Constructs a graph from OpenStreetMap data.
    ///
    /// - `input_bytes`: Bytes of an osm.pbf or osm.xml file
    /// - `osm_reader`: A callback for every OSM element read, to extract non-graph data
    /// - `post_process_graph`: A callback to remove edges and intersections after initially
    ///   importing.
    /// - `scrape_graph`: A callback to capture anything from osm2graph that's otherwise lost. It
    ///   can be stored on the `osm_reader` struct.
    /// - `profiles`: A list of named profiles. Each one assigns an access direction and cost,
    ///   given OSM tags and a Euclidean center-line. If every profile assigns `Direction::None`,
    ///   then the Road is completely excluded from the graph.
    pub fn new<R: utils::osm2graph::OsmReader>(
        input_bytes: &[u8],
        osm_reader: &mut R,
        post_process_graph: Box<dyn Fn(&mut utils::osm2graph::Graph) -> Result<()>>,
        scrape_graph: Box<dyn Fn(&mut R, &utils::osm2graph::Graph) -> Result<()>>,
        profiles: Vec<(
            String,
            Box<dyn Fn(&Tags, &LineString) -> (Direction, Duration)>,
        )>,
        timer: &mut Timer,
    ) -> Result<Graph> {
        timer.step("parse OSM and split graph");

        let mut graph = utils::osm2graph::Graph::new(
            input_bytes,
            |tags| {
                if !tags.has("highway") || tags.is("highway", "proposed") || tags.is("area", "yes")
                {
                    return false;
                }
                // Make sure at least one profile allows access
                // TODO It's weird to pass in an empty linestring
                // TODO It's inefficient to call the profiles twice
                let empty = LineString::new(Vec::new());
                profiles
                    .iter()
                    .any(|(_, profile)| profile(tags, &empty).0 != Direction::None)
            },
            osm_reader,
        )?;
        post_process_graph(&mut graph)?;
        graph.compact_ids();
        scrape_graph(osm_reader, &graph)?;

        timer.step("calculate road attributes");
        let mut roads: Vec<Road> = graph
            .edges
            .into_values()
            .map(|e| Road {
                id: RoadID(e.id.0),
                src_i: IntersectionID(e.src.0),
                dst_i: IntersectionID(e.dst.0),
                way: e.osm_way,
                node1: e.osm_node1,
                node2: e.osm_node2,
                osm_tags: e.osm_tags,
                length_meters: Euclidean.length(&e.linestring),
                linestring: e.linestring,

                access: Vec::new(),
                cost: Vec::new(),
                stops: Vec::new(),
            })
            .collect();

        // Copy all the fields
        let intersections: Vec<Intersection> = graph
            .intersections
            .into_values()
            .map(|mut i| {
                // Sort intersection roads clockwise, starting from North
                i.edges.sort_by_cached_key(|edge_id| {
                    let road = &roads[edge_id.0];
                    let bearing = bearing_from_endpoint(i.point, &road.linestring);
                    // work around that f64 is not Ord
                    debug_assert!(
                        bearing.is_finite(),
                        "Assuming bearing output is always 0...360, this shouldn't happen"
                    );
                    (bearing * 1e6) as i64
                });

                Intersection {
                    id: IntersectionID(i.id.0),
                    point: i.point,
                    node: i.osm_node,
                    roads: i.edges.into_iter().map(|e| RoadID(e.0)).collect(),
                }
            })
            .collect();

        timer.step("set up profiles");
        for road in &mut roads {
            let mut access = Vec::new();
            let mut cost = Vec::new();
            for (_, profile) in &profiles {
                let (dir, c) = profile(&road.osm_tags, &road.linestring);
                access.push(dir);
                cost.push(c);
            }
            road.access = access;
            road.cost = cost;
        }

        timer.push("building routers");
        let mut routers = Vec::new();
        let mut profile_names = BTreeMap::new();
        for (idx, (name, _)) in profiles.into_iter().enumerate() {
            timer.step(format!("for {name}"));
            routers.push(Router::new(&roads, ProfileID(idx)));

            profile_names.insert(name, ProfileID(idx));
        }
        timer.pop();

        Ok(Graph {
            roads,
            intersections,
            mercator: graph.mercator,
            profile_names,
            walking_profile_for_transit: None,
            routers,
            boundary_polygon: graph.boundary_polygon,

            gtfs: GtfsModel::empty(),
        })
    }

    /// Adds in GTFS data to the current graph. This only makes sense to call once.
    #[cfg(feature = "gtfs")]
    pub async fn setup_gtfs(
        &mut self,
        source: crate::GtfsSource,
        profile: ProfileID,
        timer: &mut Timer,
    ) -> Result<()> {
        if self.walking_profile_for_transit.is_some() {
            bail!("Can't call setup_gtfs twice");
        }
        self.walking_profile_for_transit = Some(profile);

        use crate::GtfsSource;

        timer.push("setting up GTFS");
        timer.step("parse");
        let mut gtfs = match source {
            GtfsSource::Dir(path) => GtfsModel::parse(&path, Some(&self.mercator))?,
            GtfsSource::Geomedea(url) => GtfsModel::from_geomedea(&url, &self.mercator).await?,
            GtfsSource::None => GtfsModel::empty(),
        };
        snap_stops(&mut self.roads, &mut gtfs, &self.routers[profile.0], timer);
        self.gtfs = gtfs;
        timer.pop();
        Ok(())
    }
}

#[cfg(feature = "gtfs")]
fn snap_stops(
    roads: &mut Vec<Road>,
    gtfs: &mut GtfsModel,
    foot_router: &Router,
    timer: &mut Timer,
) {
    if gtfs.stops.is_empty() {
        return;
    }

    timer.step(format!(
        "find closest roads per stop ({})",
        gtfs.stops.len()
    ));
    // TODO Make an iterator method that returns the IDs too
    for (idx, stop) in gtfs.stops.iter_mut().enumerate() {
        let stop_id = crate::gtfs::StopID(idx);
        if let Some(r) = foot_router
            .closest_road
            .nearest_neighbor(&stop.point.into())
        {
            // TODO Limit how far away we snap, or use the boundary polygon
            roads[r.data.0].stops.push(stop_id);
            stop.road = r.data;
        } else {
            // TODO Need to get rid of the stop
            error!("{stop_id:?} didn't snap to any road");
        }
    }
}

// Code copied from https://github.com/a-b-street/ltn/blob/main/backend/src/geo_helpers/mod.rs,
// without tests.
// TODO Upstream to utils or geo.
/// The bearing of the first segment of `linestring` starting from `endpoint`.
///
/// precondition: `endpoint` must be either the first or last point in `linestring`
/// precondition: `linestring` must have at least 2 coordinates
fn bearing_from_endpoint(endpoint: Point, linestring: &LineString) -> f64 {
    assert!(
        linestring.0.len() >= 2,
        "zero length roads should be filtered out"
    );
    let next_coord = if endpoint.0 == linestring.0[0] {
        linestring.0[1]
    } else if endpoint.0 == linestring.0[linestring.0.len() - 1] {
        linestring.0[linestring.0.len() - 2]
    } else {
        // I'm assuming this won't happen, but maybe it's possible,
        // e.g. to different rounding schemes.
        debug_assert!(false, "road does not terminate at intersection");
        linestring.0[1]
    };

    euclidean_bearing(endpoint.0, next_coord)
}

fn euclidean_bearing(origin: Coord, destination: Coord) -> f64 {
    (angle_of_line(Line::new(origin, destination)) + 450.0) % 360.0
}

fn angle_of_line(line: Line) -> f64 {
    (line.dy()).atan2(line.dx()).to_degrees()
}
