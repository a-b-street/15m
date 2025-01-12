use std::collections::BTreeMap;
use std::time::Duration;

use anyhow::Result;
use geo::{Euclidean, Length, LineString};
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
    /// - `profiles`: A list of named profiles. Each one assigns an access direction and cost,
    ///   given OSM tags and a Euclidean center-line. If every profile assigns `Direction::None`,
    ///   then the Road is completely excluded from the graph.
    pub fn new<R: utils::osm2graph::OsmReader>(
        input_bytes: &[u8],
        osm_reader: &mut R,
        post_process_graph: Box<dyn Fn(&mut utils::osm2graph::Graph) -> Result<()>>,
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

        timer.step("calculate road attributes");
        // Copy all the fields
        let intersections: Vec<Intersection> = graph
            .intersections
            .into_values()
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
            .into_values()
            .map(|e| Road {
                id: RoadID(e.id.0),
                src_i: IntersectionID(e.src.0),
                dst_i: IntersectionID(e.dst.0),
                way: e.osm_way,
                node1: e.osm_node1,
                node2: e.osm_node2,
                osm_tags: e.osm_tags,
                length_meters: e.linestring.length::<Euclidean>(),
                linestring: e.linestring,

                access: Vec::new(),
                cost: Vec::new(),
                stops: Vec::new(),
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
