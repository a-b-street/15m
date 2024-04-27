use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::{MapModel, RoadID};
use anyhow::Result;
use geo::{Coord, EuclideanLength};

pub fn calculate(map: &MapModel, req: Coord) -> Result<String> {
    // 1km in cm
    // TODO Use a real cost type
    let limit = 1000 * 100;
    let cost_per_road = get_costs(map, req, limit);

    // Show cost per road
    let mut features = Vec::new();
    for (r, cost) in cost_per_road {
        let mut f = geojson::Feature::from(geojson::Geometry::from(
            &map.mercator.to_wgs84(&map.roads[r.0].linestring),
        ));
        f.set_property("cost_meters", (cost as f64) / 100.0);
        features.push(f);
    }
    let gj = geojson::GeoJson::from(features);
    let x = serde_json::to_string(&gj)?;
    Ok(x)
}

fn get_costs(map: &MapModel, req: Coord, limit: usize) -> HashMap<RoadID, usize> {
    let start = map
        .closest_intersection
        .nearest_neighbor(&[req.x, req.y])
        .unwrap()
        .data;

    let mut queue: BinaryHeap<PriorityQueueItem<usize, RoadID>> = BinaryHeap::new();
    // TODO Match closest road. For now, start with all roads for the closest intersection
    for r in &map.intersections[start.0].roads {
        queue.push(PriorityQueueItem::new(0, *r));
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

        let current_road = &map.roads[current.value.0];
        for i in [current_road.src_i, current_road.dst_i] {
            for r in &map.intersections[i.0].roads {
                let cost = (100.0 * map.roads[r.0].linestring.euclidean_length()).round() as usize;
                queue.push(PriorityQueueItem::new(current.cost + cost, *r));
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
