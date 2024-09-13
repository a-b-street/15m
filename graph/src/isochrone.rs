use chrono::NaiveTime;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::time::Duration;

use utils::PriorityQueueItem;

use crate::costs::cost;
use crate::{Graph, IntersectionID, Mode, RoadID};

impl Graph {
    // TODO Doesn't account for start/end distance along roads
    /// From a list of start intersections, floods out the graph for a mode until `end_time` is
    /// reached. Returns the time needed to reach each road within that range. This query is not
    /// precise about positions along a road.
    pub fn get_costs(
        &self,
        starts: Vec<IntersectionID>,
        mode: Mode,
        public_transit: bool,
        start_time: NaiveTime,
        end_time: NaiveTime,
    ) -> HashMap<RoadID, Duration> {
        let mut visited: HashSet<IntersectionID> = HashSet::new();
        let mut cost_per_road: HashMap<RoadID, Duration> = HashMap::new();
        let mut queue: BinaryHeap<PriorityQueueItem<NaiveTime, IntersectionID>> = BinaryHeap::new();

        for start in starts {
            queue.push(PriorityQueueItem::new(start_time, start));
        }

        while let Some(current) = queue.pop() {
            if visited.contains(&current.value) {
                continue;
            }
            visited.insert(current.value);
            if current.cost > end_time {
                continue;
            }

            for r in &self.intersections[current.value.0].roads {
                let road = &self.roads[r.0];
                let total_cost = current.cost + cost(road, mode);
                cost_per_road
                    .entry(*r)
                    .or_insert((total_cost - start_time).to_std().unwrap());

                if road.src_i == current.value && road.allows_forwards(mode) {
                    queue.push(PriorityQueueItem::new(total_cost, road.dst_i));
                }
                if road.dst_i == current.value && road.allows_backwards(mode) {
                    queue.push(PriorityQueueItem::new(total_cost, road.src_i));
                }

                if public_transit {
                    for stop1 in &road.stops {
                        // Find all trips leaving from this step before the end_time
                        for next_step in self.gtfs.trips_from(
                            *stop1,
                            current.cost,
                            (end_time - current.cost).to_std().unwrap(),
                        ) {
                            // TODO Awkwardly, arrive at both intersections for the next stop's road
                            let stop2_road = &self.roads[self.gtfs.stops[next_step.stop2.0].road.0];
                            for i in [stop2_road.src_i, stop2_road.dst_i] {
                                queue.push(PriorityQueueItem::new(next_step.time2, i));
                            }
                        }
                    }
                }
            }
        }

        cost_per_road
    }
}
