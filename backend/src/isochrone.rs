use std::time::Duration;

use anyhow::Result;
use chrono::NaiveTime;
use geo::{Coord, Densify, Euclidean, Rect};
use geojson::{Feature, Geometry};
use graph::{Graph, ProfileID, Timer};
use serde::Deserialize;
use utils::Grid;

use crate::Amenities;

#[derive(Deserialize)]
pub enum Style {
    Roads,
    Grid,
    Contours,
}

pub enum Source {
    Single(Coord),
    FromAmenity(String),
}

pub fn calculate(
    graph: &Graph,
    amenities: &Amenities,
    source: Source,
    profile: ProfileID,
    style: Style,
    public_transit: bool,
    start_time: NaiveTime,
    limit: Duration,
    mut timer: Timer,
) -> Result<String> {
    let mut starts = Vec::new();
    match source {
        Source::Single(pt) => {
            starts.push(graph.snap_to_road(pt, profile).intersection);
        }
        Source::FromAmenity(kind) => {
            for (r, lists) in amenities.per_road.iter().enumerate() {
                for a in &lists[profile.0] {
                    let amenity = &amenities.amenities[a.0];
                    if amenity.kind == kind {
                        let road = &graph.roads[r];
                        // TODO Which intersection is closer? Just start from either
                        starts.push(road.src_i);
                        starts.push(road.dst_i);
                    }
                }
            }
            starts.sort();
            starts.dedup();
            if starts.is_empty() {
                bail!("No amenities of kind {kind}");
            }
        }
    }

    timer.step("get_costs");
    let cost_per_road = graph.get_costs(
        starts,
        profile,
        public_transit,
        start_time,
        start_time + limit,
    );
    timer.push("render to GJ");

    // Show reached amenities
    let mut features = Vec::new();
    for (r, _) in &cost_per_road {
        for a in &amenities.per_road[r.0][profile.0] {
            features.push(amenities.amenities[a.0].to_gj(&graph.mercator));
        }
    }

    match style {
        Style::Roads => {
            for (r, cost) in cost_per_road {
                let mut f = Feature::from(Geometry::from(
                    &graph.mercator.to_wgs84(&graph.roads[r.0].linestring),
                ));
                f.set_property("cost_seconds", cost.as_secs());
                features.push(f);
            }
        }
        Style::Grid | Style::Contours => {
            timer.step("make grid");
            // Grid values are cost in seconds
            let mut grid: Grid<f64> = Grid::new(
                (graph.mercator.width / RESOLUTION_M).ceil() as usize,
                (graph.mercator.height / RESOLUTION_M).ceil() as usize,
                0.0,
            );

            for (r, cost) in cost_per_road {
                for pt in graph.roads[r.0]
                    .linestring
                    .densify::<Euclidean>(RESOLUTION_M / 2.0)
                    .0
                {
                    let grid_idx = grid.idx(
                        (pt.x / RESOLUTION_M) as usize,
                        (pt.y / RESOLUTION_M) as usize,
                    );
                    // If there are overlapping grid cells (bridges, tunnels, precision), just blindly
                    // clobber
                    grid.data[grid_idx] = cost.as_secs_f64();
                }
            }

            if matches!(style, Style::Grid) {
                features.extend(render_grid(graph, grid));
            } else {
                features.extend(render_contours(graph, grid));
            }
        }
    }
    timer.pop();

    let gj = geojson::GeoJson::from(features);
    let x = serde_json::to_string(&gj)?;
    timer.done();

    Ok(x)
}

const RESOLUTION_M: f64 = 100.0;

fn render_contours(graph: &Graph, grid: Grid<f64>) -> Vec<Feature> {
    let smooth = false;
    let contour_builder = contour::ContourBuilder::new(grid.width, grid.height, smooth)
        .x_step(RESOLUTION_M)
        .y_step(RESOLUTION_M);
    let thresholds = vec![3. * 60., 6. * 60., 9. * 60., 12. * 60., 15. * 60.];

    let mut features = Vec::new();
    for band in contour_builder.isobands(&grid.data, &thresholds).unwrap() {
        let mut f = Feature::from(Geometry::from(&graph.mercator.to_wgs84(band.geometry())));
        f.set_property("min_seconds", band.min_v());
        f.set_property("max_seconds", band.max_v());
        features.push(f);
    }
    features
}

fn render_grid(graph: &Graph, grid: Grid<f64>) -> Vec<Feature> {
    let mut features = Vec::new();
    for x in 0..grid.width {
        for y in 0..grid.height {
            let value = grid.data[grid.idx(x, y)];
            if value == 0.0 {
                continue;
            }

            let rect = Rect::new(
                Coord {
                    x: (x as f64) * RESOLUTION_M,
                    y: (y as f64) * RESOLUTION_M,
                },
                Coord {
                    x: ((x + 1) as f64) * RESOLUTION_M,
                    y: ((y + 1) as f64) * RESOLUTION_M,
                },
            )
            .to_polygon();
            let mut f = Feature::from(Geometry::from(&graph.mercator.to_wgs84(&rect)));
            let step = 3.0 * 60.0;
            let min = step * (value / step).floor();
            f.set_property("min_seconds", min);
            f.set_property("max_seconds", min + step);
            features.push(f);
        }
    }

    features
}
