use std::collections::{BinaryHeap, HashMap};
use std::time::Duration;

use anyhow::Result;
use geo::{Coord, EuclideanLength};

use crate::graph::{Graph, Mode, Road, RoadID};
use crate::priority_queue::PriorityQueueItem;

pub fn calculate(graph: &Graph, req: Coord, mode: Mode) -> Result<String> {
    // 15 minutes
    let limit = Duration::from_secs(15 * 60);

    let cost_per_road = get_costs(graph, req, mode, limit);

    // Show cost per road
    let mut features = Vec::new();
    for (r, cost) in cost_per_road {
        let mut f = geojson::Feature::from(geojson::Geometry::from(
            &graph.mercator.to_wgs84(&graph.roads[r.0].linestring),
        ));
        f.set_property("cost_seconds", cost.as_secs());
        features.push(f);

        for a in &graph.roads[r.0].amenities[mode] {
            features.push(graph.amenities[a.0].to_gj(&graph.mercator));
        }
    }
    let gj = geojson::GeoJson::from(features);
    let x = serde_json::to_string(&gj)?;
    Ok(x)
}

fn get_costs(graph: &Graph, req: Coord, mode: Mode, limit: Duration) -> HashMap<RoadID, Duration> {
    let start = graph.closest_intersection[mode]
        .nearest_neighbor(&[req.x, req.y])
        .unwrap()
        .data;

    let mut queue: BinaryHeap<PriorityQueueItem<Duration, RoadID>> = BinaryHeap::new();
    // TODO Match closest road. For now, start with all roads for the closest intersection
    // TODO Think through directions for this initial case. Going by road is strange.
    for road in graph.roads_per_intersection(start, mode) {
        queue.push(PriorityQueueItem::new(Duration::ZERO, road.id));
    }

    let mut cost_per_road: HashMap<RoadID, Duration> = HashMap::new();
    while let Some(current) = queue.pop() {
        if cost_per_road.contains_key(&current.value) {
            continue;
        }
        if current.cost > limit {
            continue;
        }
        cost_per_road.insert(current.value, current.cost);

        let current_road = &graph.roads[current.value.0];
        // TODO Think through how this search should work with directions. This is assuming
        // incorrectly we're starting from src_i.
        let mut endpoints = Vec::new();
        if current_road.allows_forwards(mode) {
            endpoints.push(current_road.dst_i);
        }
        if current_road.allows_backwards(mode) {
            endpoints.push(current_road.src_i);
        }

        for i in endpoints {
            for road in graph.roads_per_intersection(i, mode) {
                queue.push(PriorityQueueItem::new(
                    current.cost + cost(road, mode),
                    road.id,
                ));
            }
        }
    }

    cost_per_road
}

fn cost(road: &Road, mode: Mode) -> Duration {
    // TODO Configurable
    // 10 mph
    let max_bicycle_speed = 4.4704;
    // 3 mph
    let max_foot_speed = 1.34112;

    // All speeds are meters/second, so the units work out
    let distance = road.linestring.euclidean_length();
    match mode {
        Mode::Car => Duration::from_secs_f64(distance / road.max_speed),
        // TODO Use elevation and other more detailed things
        Mode::Bicycle => Duration::from_secs_f64(distance / max_bicycle_speed),
        Mode::Foot => Duration::from_secs_f64(distance / max_foot_speed),
    }
}
