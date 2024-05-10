use std::collections::{BinaryHeap, HashMap};
use std::time::Duration;

use anyhow::Result;
use geo::{Coord, Densify};
use utils::{Grid, PriorityQueueItem};

use crate::costs::cost;
use crate::graph::{Graph, Mode, RoadID};
use crate::timer::Timer;

pub fn calculate(graph: &Graph, req: Coord, mode: Mode, contours: bool) -> Result<String> {
    // 15 minutes
    let limit = Duration::from_secs(15 * 60);

    let mut timer = Timer::new("isochrone request");
    timer.step("get_costs");
    let cost_per_road = get_costs(graph, req, mode, limit);
    timer.push("render to GJ");

    // Show cost per road
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
