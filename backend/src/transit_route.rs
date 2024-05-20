use std::collections::hash_map::Entry;
use std::collections::{BinaryHeap, HashMap};
use std::time::Duration;

use anyhow::{bail, Result};
use chrono::NaiveTime;
use geo::LineString;
use geojson::{Feature, GeoJson, Geometry};
use utils::PriorityQueueItem;

use crate::costs::cost;
use crate::graph::{Graph, IntersectionID, Mode, RoadID};
use crate::gtfs::{StopID, TripID};

pub fn route(graph: &Graph, start: IntersectionID, end: IntersectionID) -> Result<String> {
    // TODO We'll need a start time too (and a day of the week)
    let start_time = NaiveTime::from_hms_opt(7, 0, 0).unwrap();
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

    let mut queue: BinaryHeap<PriorityQueueItem<NaiveTime, IntersectionID>> = BinaryHeap::new();
    queue.push(PriorityQueueItem::new(start_time, start));

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
                    PathStep::Transit { stop1, stop2, .. } => {
                        features.push(Feature::from(Geometry::from(&graph.mercator.to_wgs84(
                            &LineString::new(vec![
                                graph.gtfs.stops[stop1].point.into(),
                                graph.gtfs.stops[stop2].point.into(),
                            ]),
                        ))));
                    }
                }
                at = *prev_i;
            }
        }

        for r in &graph.intersections[current.value.0].roads {
            let road = &graph.roads[r.0];

            // Handle walking to the other end of the road
            let total_cost = current.cost + cost(road, Mode::Foot);
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

            // Use transit!
            for stop1 in &road.stops {
                // Find all trips leaving from this step in the next 30 minutes
                // TODO Figure out how to prune that search time better
                for next_step in
                    graph
                        .gtfs
                        .trips_from(stop1, current.cost, Duration::from_secs(30 * 60))
                {
                    // TODO Here's the awkwardness -- arrive at both the intersections for that
                    // road
                    let stop2_road = &graph.roads[graph.gtfs.stops[&next_step.stop2].road.0];
                    for i in [stop2_road.src_i, stop2_road.dst_i] {
                        if let Entry::Vacant(entry) = backrefs.entry(i) {
                            entry.insert((
                                current.value,
                                PathStep::Transit {
                                    stop1: stop1.clone(),
                                    trip: next_step.trip.clone(),
                                    stop2: next_step.stop2.clone(),
                                },
                            ));
                            queue.push(PriorityQueueItem::new(next_step.time2, i));
                        }
                    }
                }
            }
        }
    }

    bail!("No path found");
}

enum PathStep {
    Road(RoadID),
    Transit {
        stop1: StopID,
        trip: TripID,
        stop2: StopID,
    },
}
