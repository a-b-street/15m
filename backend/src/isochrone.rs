use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use anyhow::Result;
use geo::{Coord, EuclideanLength};

use crate::graph::{Graph, Mode, RoadID};

pub fn calculate(graph: &Graph, req: Coord, mode: Mode) -> Result<String> {
    // 1km in cm
    // TODO Use a real cost type
    let limit = 1000 * 100;
    let cost_per_road = get_costs(graph, req, mode, limit);

    // Show cost per road
    let mut features = Vec::new();
    for (r, cost) in cost_per_road {
        let mut f = geojson::Feature::from(geojson::Geometry::from(
            &graph.mercator.to_wgs84(&graph.roads[r.0].linestring),
        ));
        f.set_property("cost_meters", (cost as f64) / 100.0);
        features.push(f);
    }
    let gj = geojson::GeoJson::from(features);
    let x = serde_json::to_string(&gj)?;
    Ok(x)
}

fn get_costs(graph: &Graph, req: Coord, mode: Mode, limit: usize) -> HashMap<RoadID, usize> {
    // TODO This needs to be per mode
    let start = graph
        .closest_intersection
        .nearest_neighbor(&[req.x, req.y])
        .unwrap()
        .data;

    let mut queue: BinaryHeap<PriorityQueueItem<usize, RoadID>> = BinaryHeap::new();
    // TODO Match closest road. For now, start with all roads for the closest intersection
    // TODO Think through directions for this initial case. Going by road is strange.
    for road in graph.roads_per_intersection(start, mode) {
        queue.push(PriorityQueueItem::new(0, road.id));
    }

    let mut cost_per_road: HashMap<RoadID, usize> = HashMap::new();
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
                // TODO Different cost per mode
                let cost = (100.0 * road.linestring.euclidean_length()).round() as usize;
                queue.push(PriorityQueueItem::new(current.cost + cost, road.id));
            }
        }
    }

    cost_per_road
}

/// Use with `BinaryHeap`. Since it's a max-heap, reverse the comparison to get the smallest cost
/// first.
#[derive(PartialEq, Eq, Clone)]
struct PriorityQueueItem<K, V> {
    pub cost: K,
    pub value: V,
}

impl<K, V> PriorityQueueItem<K, V> {
    fn new(cost: K, value: V) -> Self {
        Self { cost, value }
    }
}

impl<K: Ord, V: Ord> PartialOrd for PriorityQueueItem<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord, V: Ord> Ord for PriorityQueueItem<K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = other.cost.cmp(&self.cost);
        if ord != Ordering::Equal {
            return ord;
        }
        // The tie-breaker is arbitrary, based on the value
        self.value.cmp(&other.value)
    }
}
