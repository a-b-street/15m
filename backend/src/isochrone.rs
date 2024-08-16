use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use chrono::NaiveTime;
use geo::{Coord, Densify};
use utils::Grid;

use crate::graph::{Graph, Mode, RoadID};
use crate::timer::Timer;

pub fn calculate(
    graph: &Graph,
    req: Coord,
    mode: Mode,
    contours: bool,
    public_transit: bool,
    start_time: NaiveTime,
    limit: Duration,
    mut timer: Timer,
) -> Result<String> {
    timer.step("get_costs");
    let cost_per_road = graph.get_costs(
        vec![graph.snap_to_road(req, mode).intersection],
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
