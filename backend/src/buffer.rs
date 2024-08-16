use std::collections::HashSet;
use std::time::Duration;

use anyhow::Result;
use chrono::NaiveTime;
use geojson::{Feature, GeoJson, Geometry};

use crate::graph::{Graph, Mode, PathStep};

pub fn buffer_route(
    graph: &Graph,
    mode: Mode,
    steps: Vec<PathStep>,
    start_time: NaiveTime,
    limit: Duration,
) -> Result<String> {
    let mut features = Vec::new();
    let mut route_roads = HashSet::new();
    let mut starts = HashSet::new();
    for step in steps {
        if let PathStep::Road { road, .. } = step {
            route_roads.insert(road);
            let road = &graph.roads[road.0];
            starts.insert(road.src_i);
            starts.insert(road.dst_i);

            // TODO Doesn't handle the exact start/end
            let mut f = Feature::from(Geometry::from(&graph.mercator.to_wgs84(&road.linestring)));
            f.set_property("kind", "route");
            features.push(f);
        }
    }

    let public_transit = false; // TODO
    let cost_per_road = graph.get_costs(
        starts.into_iter().collect(),
        mode,
        public_transit,
        start_time,
        start_time + limit,
    );
    for (r, cost) in cost_per_road {
        if !route_roads.contains(&r) {
            let mut f = Feature::from(Geometry::from(
                &graph.mercator.to_wgs84(&graph.roads[r.0].linestring),
            ));
            f.set_property("kind", "buffer");
            f.set_property("cost_seconds", cost.as_secs());
            features.push(f);
        }
    }

    Ok(serde_json::to_string(&GeoJson::from(features))?)
}
