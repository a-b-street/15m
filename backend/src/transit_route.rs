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
use crate::timer::Timer;

pub fn route(
    graph: &Graph,
    start: IntersectionID,
    end: IntersectionID,
    // TODO Parameterizing this function gets messy, but splitting into two separate doesn't seem
    // like a good idea yet
    debug_search: bool,
    mut timer: Timer,
) -> Result<String> {
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

    let mut backrefs: HashMap<IntersectionID, Backreference> = HashMap::new();

    // If we're debugging the search process, just remember the order of visited nodes
    let mut search_record: Vec<IntersectionID> = Vec::new();

    timer.step("dijkstra");
    let mut queue: BinaryHeap<PriorityQueueItem<NaiveTime, IntersectionID>> = BinaryHeap::new();
    queue.push(PriorityQueueItem::new(start_time, start));

    while let Some(current) = queue.pop() {
        if current.value == end {
            if debug_search {
                return render_debug(search_record, backrefs, graph, timer);
            } else {
                return render_path(backrefs, graph, start, end, timer);
            }
        }
        if debug_search {
            search_record.push(current.value);
        }

        for r in &graph.intersections[current.value.0].roads {
            let road = &graph.roads[r.0];

            // Handle walking to the other end of the road
            let total_cost = current.cost + cost(road, Mode::Foot);
            if road.src_i == current.value && road.allows_forwards(Mode::Foot) {
                if let Entry::Vacant(entry) = backrefs.entry(road.dst_i) {
                    entry.insert(Backreference {
                        src_i: current.value,
                        step: PathStep::Road {
                            road: *r,
                            forwards: true,
                        },
                        time1: current.cost,
                        time2: total_cost,
                    });
                    queue.push(PriorityQueueItem::new(total_cost, road.dst_i));
                }
            } else if road.dst_i == current.value && road.allows_backwards(Mode::Foot) {
                if let Entry::Vacant(entry) = backrefs.entry(road.src_i) {
                    entry.insert(Backreference {
                        src_i: current.value,
                        step: PathStep::Road {
                            road: *r,
                            forwards: false,
                        },
                        time1: current.cost,
                        time2: total_cost,
                    });
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
                        .trips_from(*stop1, current.cost, Duration::from_secs(30 * 60))
                {
                    // TODO Here's the awkwardness -- arrive at both the intersections for that
                    // road
                    let stop2_road = &graph.roads[graph.gtfs.stops[next_step.stop2.0].road.0];
                    for i in [stop2_road.src_i, stop2_road.dst_i] {
                        if let Entry::Vacant(entry) = backrefs.entry(i) {
                            entry.insert(Backreference {
                                src_i: current.value,
                                step: PathStep::Transit {
                                    stop1: *stop1,
                                    trip: next_step.trip,
                                    stop2: next_step.stop2,
                                },
                                time1: next_step.time1,
                                time2: next_step.time2,
                            });
                            queue.push(PriorityQueueItem::new(next_step.time2, i));
                        }
                    }
                }
            }
        }
    }

    bail!("No path found");
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

enum PathStep {
    Road {
        road: RoadID,
        forwards: bool,
    },
    Transit {
        stop1: StopID,
        trip: TripID,
        stop2: StopID,
    },
}

fn render_path(
    mut backrefs: HashMap<IntersectionID, Backreference>,
    graph: &Graph,
    start: IntersectionID,
    end: IntersectionID,
    mut timer: Timer,
) -> Result<String> {
    timer.step("render");

    // Just get PathSteps in order first (Step, time1, time2)
    let mut steps: Vec<(PathStep, NaiveTime, NaiveTime)> = Vec::new();
    let mut at = end;
    loop {
        if at == start {
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
