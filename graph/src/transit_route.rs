use std::collections::hash_map::Entry;
use std::collections::{BinaryHeap, HashMap};
use std::time::Duration;

use anyhow::{bail, Result};
use chrono::NaiveTime;
use geo::{Distance, Euclidean, LineString};
use geojson::{Feature, GeoJson, Geometry};
use utils::PriorityQueueItem;

use crate::{Graph, IntersectionID, PathStep, Position, Timer};

impl Graph {
    pub fn transit_route_gj(
        &self,
        start: Position,
        end: Position,
        // TODO Parameterizing this function gets messy, but splitting into two separate doesn't seem
        // like a good idea yet
        debug_search: bool,
        use_heuristic: bool,
        start_time: NaiveTime,
        mut timer: Timer,
    ) -> Result<String> {
        if start == end {
            bail!("start = end");
        }
        let Some(profile) = self.walking_profile_for_transit else {
            bail!("public transit hasn't been set up");
        };
        // TODO Handle start.road == end.road case. Share code somewhere.

        let end_pt = self.intersections[end.intersection.0].point;
        // TODO Share constant properly
        // TODO Think through if this is admissible and/or consistent
        let heuristic = |i: IntersectionID| {
            if use_heuristic {
                Duration::from_secs_f64(
                    Euclidean.distance(self.intersections[i.0].point, end_pt) / 1.34112,
                )
            } else {
                Duration::ZERO
            }
        };

        // TODO stops are associated with roads, so the steps using transit are going to look a little
        // weird / be a little ambiguous
        //
        // or rethink the nodes and edges in the graph. nodes are pathsteps -- a road in some direction
        // or a transit thing. an edge is a turn or a transition to/from transit

        let mut backrefs: HashMap<IntersectionID, Backreference> = HashMap::new();

        // If we're debugging the search process, just remember the order of visited nodes
        let mut search_record: Vec<IntersectionID> = Vec::new();

        timer.step("dijkstra");
        // Store the actual cost/time to reach somewhere as the item. Include a heuristic
        let mut queue: BinaryHeap<PriorityQueueItem<NaiveTime, (IntersectionID, NaiveTime)>> =
            BinaryHeap::new();
        queue.push(PriorityQueueItem::new(
            start_time + heuristic(start.intersection),
            (start.intersection, start_time),
        ));

        while let Some(current) = queue.pop() {
            // Don't use current.cost, since it might include a heuristic
            let (current_i, current_time) = current.value;
            if current_i == end.intersection {
                if debug_search {
                    return render_debug(search_record, backrefs, self, timer);
                } else {
                    return render_path(backrefs, self, start, end, timer);
                }
            }
            if debug_search {
                search_record.push(current_i);
            }

            for r in &self.intersections[current_i.0].roads {
                let road = &self.roads[r.0];

                // Handle walking to the other end of the road
                let total_cost = current_time + road.cost[profile.0];
                if road.src_i == current_i && road.allows_forwards(profile) {
                    if let Entry::Vacant(entry) = backrefs.entry(road.dst_i) {
                        entry.insert(Backreference {
                            src_i: current_i,
                            step: PathStep::Road {
                                road: *r,
                                forwards: true,
                            },
                            time1: current_time,
                            time2: total_cost,
                        });
                        queue.push(PriorityQueueItem::new(
                            total_cost + heuristic(road.dst_i),
                            (road.dst_i, total_cost),
                        ));
                    }
                } else if road.dst_i == current_i && road.allows_backwards(profile) {
                    if let Entry::Vacant(entry) = backrefs.entry(road.src_i) {
                        entry.insert(Backreference {
                            src_i: current_i,
                            step: PathStep::Road {
                                road: *r,
                                forwards: false,
                            },
                            time1: current_time,
                            time2: total_cost,
                        });
                        queue.push(PriorityQueueItem::new(
                            total_cost + heuristic(road.src_i),
                            (road.src_i, total_cost),
                        ));
                    }
                }

                // Use transit!
                for stop1 in &road.stops {
                    // Find all trips leaving from this step in the next 30 minutes
                    // TODO Figure out how to prune that search time better
                    for next_step in
                        self.gtfs
                            .trips_from(*stop1, current_time, Duration::from_secs(30 * 60))
                    {
                        // TODO Here's the awkwardness -- arrive at both the intersections for that
                        // road
                        let stop2_road = &self.roads[self.gtfs.stops[next_step.stop2.0].road.0];
                        for i in [stop2_road.src_i, stop2_road.dst_i] {
                            if let Entry::Vacant(entry) = backrefs.entry(i) {
                                entry.insert(Backreference {
                                    src_i: current_i,
                                    step: PathStep::Transit {
                                        stop1: *stop1,
                                        trip: next_step.trip,
                                        stop2: next_step.stop2,
                                    },
                                    time1: next_step.time1,
                                    time2: next_step.time2,
                                });
                                queue.push(PriorityQueueItem::new(
                                    next_step.time2 + heuristic(i),
                                    (i, next_step.time2),
                                ));
                            }
                        }
                    }
                }
            }
        }

        bail!("No path found");
    }
}

// How'd we get somewhere?
struct Backreference {
    // Where were we at the beginning of this step?
    src_i: IntersectionID,
    step: PathStep,
    // When'd we start this step?
    time1: NaiveTime,
    // When'd we finish?
    time2: NaiveTime,
}

fn render_path(
    mut backrefs: HashMap<IntersectionID, Backreference>,
    graph: &Graph,
    start: Position,
    end: Position,
    mut timer: Timer,
) -> Result<String> {
    timer.step("render");

    // Just get PathSteps in order first (Step, time1, time2)
    let mut steps: Vec<(PathStep, NaiveTime, NaiveTime)> = Vec::new();
    let mut at = end.intersection;
    loop {
        if at == start.intersection {
            break;
        }
        let backref = backrefs.remove(&at).unwrap();
        steps.push((backref.step, backref.time1, backref.time2));
        at = backref.src_i;
    }
    steps.reverse();

    // Assemble PathSteps into features. Group road and transit steps together
    let mut features = Vec::new();
    for chunk in steps.chunk_by(|a, b| match (&a.0, &b.0) {
        (PathStep::Road { .. }, PathStep::Road { .. }) => true,
        (PathStep::Transit { trip: trip1, .. }, PathStep::Transit { trip: trip2, .. }) => {
            trip1 == trip2
        }
        _ => false,
    }) {
        let mut pts = Vec::new();
        let mut num_stops = 0;
        let mut trip_id = None;
        for (step, _, _) in chunk {
            match step {
                PathStep::Road { road, forwards } => {
                    let road = &graph.roads[road.0];
                    if *forwards {
                        pts.extend(road.linestring.0.clone());
                    } else {
                        let mut rev = road.linestring.0.clone();
                        rev.reverse();
                        pts.extend(rev);
                    }
                }
                PathStep::Transit { stop1, stop2, trip } => {
                    trip_id = Some(trip);
                    num_stops += 1;
                    pts.push(graph.gtfs.stops[stop1.0].point.into());
                    pts.push(graph.gtfs.stops[stop2.0].point.into());
                }
            }
        }
        pts.dedup();

        let mut f = Feature::from(Geometry::from(
            &graph.mercator.to_wgs84(&LineString::new(pts)),
        ));
        f.set_property("time1", chunk[0].1.to_string());
        f.set_property("time2", chunk.last().unwrap().2.to_string());

        if let Some(trip) = trip_id {
            f.set_property("kind", "transit");
            f.set_property("trip", trip.0);
            f.set_property(
                "route",
                graph.gtfs.routes[graph.gtfs.trips[trip.0].route.0].describe(),
            );
            f.set_property("num_stops", num_stops);
        } else {
            f.set_property("kind", "road");
        }
        features.push(f);
    }
    timer.done();
    Ok(serde_json::to_string(&GeoJson::from(features))?)
}

fn render_debug(
    search_record: Vec<IntersectionID>,
    mut backrefs: HashMap<IntersectionID, Backreference>,
    graph: &Graph,
    mut timer: Timer,
) -> Result<String> {
    timer.step("render");
    // Create a FeatureCollection with the nodes searched, in order. Pairs of linestrings (a step
    // to get somewhere) and points (for the intersection)
    let mut features = Vec::new();
    // Skip the first node, because it'll have no backreference
    for i in search_record.into_iter().skip(1) {
        let backref = backrefs.remove(&i).unwrap();
        match backref.step {
            PathStep::Road { road, .. } => {
                let mut f = Feature::from(Geometry::from(
                    &graph.mercator.to_wgs84(&graph.roads[road.0].linestring),
                ));
                f.set_property("kind", "road");
                features.push(f);
            }
            PathStep::Transit { stop1, stop2, .. } => {
                let mut f = Feature::from(Geometry::from(&graph.mercator.to_wgs84(
                    &LineString::new(vec![
                        graph.gtfs.stops[stop1.0].point.into(),
                        graph.gtfs.stops[stop2.0].point.into(),
                    ]),
                )));
                f.set_property("kind", "transit");
                features.push(f);
            }
        }

        let mut f = Feature::from(Geometry::from(
            &graph.mercator.to_wgs84(&graph.intersections[i.0].point),
        ));
        f.set_property("time", backref.time2.to_string());
        features.push(f);
    }

    let json = serde_json::to_string(&GeoJson::from(features))?;
    timer.done();
    Ok(json)
}
