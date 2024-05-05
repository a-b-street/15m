use std::collections::{BinaryHeap, HashMap};
use std::time::Duration;

use anyhow::Result;
use geo::Coord;
use web_time::Instant;

use crate::costs::cost;
use crate::graph::{Graph, Mode, RoadID};
use crate::priority_queue::PriorityQueueItem;

pub fn calculate(graph: &Graph, req: Coord, mode: Mode) -> Result<String> {
    // 15 minutes
    let limit = Duration::from_secs(15 * 60);

    let t1 = Instant::now();
    let cost_per_road = get_costs(graph, req, mode, limit);
    let t2 = Instant::now();

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
    let t3 = Instant::now();

    info!("Total backend isochrone time: {:?}", t3 - t1);
    for (label, dt) in [("get_costs", t2 - t1), ("to GJ", t3 - t2)] {
        info!("  {label} took {dt:?}");
    }

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
