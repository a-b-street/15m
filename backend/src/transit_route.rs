use std::collections::hash_map::Entry;
use std::collections::{BinaryHeap, HashMap};
use std::time::Duration;

use anyhow::{bail, Result};
use geojson::GeoJson;
use utils::PriorityQueueItem;

use crate::costs::cost;
use crate::graph::{Graph, IntersectionID, Mode, RoadID};

// TODO We'll need a start time too (and a day of the week)
pub fn route(graph: &Graph, start: IntersectionID, end: IntersectionID) -> Result<String> {
    if start == end {
        bail!("start = end");
    }

    // Dijkstra implementation just for walking

    // TODO stops are associated with roads, so the steps using transit are going to look a little
    // weird / be a little ambiguous
    //
    // or rethink the nodes and edges in the graph. nodes are pathsteps -- a road in some direction
    // or a transit thing. an edge is a turn or a transition to/from transit
    let mut backrefs: HashMap<IntersectionID, (IntersectionID, PathStep)> = HashMap::new();

    let mut queue: BinaryHeap<PriorityQueueItem<Duration, IntersectionID>> = BinaryHeap::new();
    queue.push(PriorityQueueItem::new(Duration::ZERO, start));

    info!("just sanity checking, we're here");

    while let Some(current) = queue.pop() {
        if current.value == end {
            // Found the path
            // TODO Ideally glue together one LineString
            let mut features = Vec::new();
            let mut at = current.value;
            loop {
                if at == start {
                    return Ok(serde_json::to_string(&GeoJson::from(features))?);
                }
                let (prev_i, step) = &backrefs[&at];
                match step {
                    PathStep::Road(r) => {
                        features.push(graph.roads[r.0].to_gj(&graph.mercator));
                    }
                }
                at = *prev_i;
            }
        }

        for r in &graph.intersections[current.value.0].roads {
            let road = &graph.roads[r.0];
            let total_cost = current.cost + cost(road, Mode::Foot);
            // Go to the other side, if we're allowed to
            if road.src_i == current.value && road.allows_forwards(Mode::Foot) {
                if let Entry::Vacant(entry) = backrefs.entry(road.dst_i) {
                    entry.insert((current.value, PathStep::Road(*r)));
                    queue.push(PriorityQueueItem::new(total_cost, road.dst_i));
                }
            } else if road.dst_i == current.value && road.allows_backwards(Mode::Foot) {
                if let Entry::Vacant(entry) = backrefs.entry(road.src_i) {
                    entry.insert((current.value, PathStep::Road(*r)));
                    queue.push(PriorityQueueItem::new(total_cost, road.src_i));
                }
            }
        }
    }

    bail!("No path found");
}

enum PathStep {
    Road(RoadID),
}
