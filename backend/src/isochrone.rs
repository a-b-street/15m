use std::collections::{BinaryHeap, HashMap, HashSet};
use std::time::Duration;

use anyhow::Result;
use chrono::NaiveTime;
use geo::{Coord, Densify};
use utils::{Grid, PriorityQueueItem};

use crate::costs::cost;
use crate::graph::{Graph, IntersectionID, Mode, RoadID};
use crate::timer::Timer;

pub fn calculate(
    graph: &Graph,
    req: Coord,
    mode: Mode,
    contours: bool,
    public_transit: bool,
    start_time: NaiveTime,
    mut timer: Timer,
) -> Result<String> {
    // 15 minutes
    let limit = Duration::from_secs(15 * 60);

    timer.step("get_costs");
    let cost_per_road = get_costs(
        graph,
        req,
        mode,
        public_transit,
        start_time,
        start_time + limit,
    );
    timer.push("render to GJ");

    // Show reached amenities
    let mut features = Vec::new();
    for (r, _) in &cost_per_road {
        for a in &graph.roads[r.0].amenities[mode] {
            features.push(graph.amenities[a.0].to_gj(&graph.mercator));
        }
    }

    if contours {
        features.extend(make_contours(graph, cost_per_road, &mut timer));
    } else {
        for (r, cost) in cost_per_road {
            let mut f = geojson::Feature::from(geojson::Geometry::from(
                &graph.mercator.to_wgs84(&graph.roads[r.0].linestring),
            ));
            f.set_property("cost_seconds", cost.as_secs());
            features.push(f);
        }
    }
    timer.pop();

    let gj = geojson::GeoJson::from(features);
    let x = serde_json::to_string(&gj)?;
    timer.done();

    Ok(x)
}

pub fn get_costs(
    graph: &Graph,
    req: Coord,
    mode: Mode,
    public_transit: bool,
    start_time: NaiveTime,
    end_time: NaiveTime,
) -> HashMap<RoadID, Duration> {
    let start = graph.closest_intersection(req, mode);

    let mut visited: HashSet<IntersectionID> = HashSet::new();
    let mut cost_per_road: HashMap<RoadID, Duration> = HashMap::new();
    let mut queue: BinaryHeap<PriorityQueueItem<NaiveTime, IntersectionID>> = BinaryHeap::new();

    queue.push(PriorityQueueItem::new(start_time, start));

    while let Some(current) = queue.pop() {
        if visited.contains(&current.value) {
            continue;
        }
        visited.insert(current.value);
        if current.cost > end_time {
            continue;
        }

        for r in &graph.intersections[current.value.0].roads {
            let road = &graph.roads[r.0];
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
                    for next_step in graph.gtfs.trips_from(
                        *stop1,
                        current.cost,
                        (end_time - current.cost).to_std().unwrap(),
                    ) {
                        // TODO Awkwardly, arrive at both intersections for the next stop's road
                        let stop2_road = &graph.roads[graph.gtfs.stops[next_step.stop2.0].road.0];
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

const RESOLUTION_M: f64 = 100.0;

fn make_contours(
    graph: &Graph,
    cost_per_road: HashMap<RoadID, Duration>,
    timer: &mut Timer,
) -> Vec<geojson::Feature> {
    timer.step("make grid");
    // Grid values are cost in seconds
    let mut grid: Grid<f64> = Grid::new(
        (graph.mercator.width / RESOLUTION_M).ceil() as usize,
        (graph.mercator.height / RESOLUTION_M).ceil() as usize,
        0.0,
    );

    for (r, cost) in cost_per_road {
        for pt in graph.roads[r.0].linestring.densify(RESOLUTION_M / 2.0).0 {
            let grid_idx = grid.idx(
                (pt.x / RESOLUTION_M) as usize,
                (pt.y / RESOLUTION_M) as usize,
            );
            // If there are overlapping grid cells (bridges, tunnels, precision), just blindly
            // clobber
            grid.data[grid_idx] = cost.as_secs_f64();
        }
    }

    timer.step("make contours");
    let smooth = false;
    let contour_builder = contour::ContourBuilder::new(grid.width, grid.height, smooth)
        .x_step(RESOLUTION_M)
        .y_step(RESOLUTION_M);
    let thresholds = vec![3. * 60., 6. * 60., 9. * 60., 12. * 60., 15. * 60.];

    let mut features = Vec::new();
    for band in contour_builder.isobands(&grid.data, &thresholds).unwrap() {
        let mut f = geojson::Feature::from(geojson::Geometry::from(
            &graph.mercator.to_wgs84(band.geometry()),
        ));
        f.set_property("min_seconds", band.min_v());
        f.set_property("max_seconds", band.max_v());
        features.push(f);
    }
    features
}
