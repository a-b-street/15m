use std::collections::HashSet;
use std::time::Duration;

use anyhow::Result;
use chrono::NaiveTime;
use geo::{Area, BooleanOps, ConvexHull, Coord, LineString, MultiPolygon, Polygon};
use geojson::{Feature, FeatureCollection, Geometry};

use crate::graph::{Graph, Mode, PathStep};

pub fn buffer_route(
    graph: &Graph,
    mode: Mode,
    steps: Vec<PathStep>,
    start_time: NaiveTime,
    limit: Duration,
) -> Result<String> {
    let mut features = Vec::new();
    let mut route_roads = HashSet::new();
    let mut starts = HashSet::new();
    for step in steps {
        if let PathStep::Road { road, .. } = step {
            route_roads.insert(road);
            let road = &graph.roads[road.0];
            starts.insert(road.src_i);
            starts.insert(road.dst_i);

            // TODO Doesn't handle the exact start/end, or gluing things together
            let mut f = Feature::from(Geometry::from(&graph.mercator.to_wgs84(&road.linestring)));
            f.set_property("kind", "route");
            features.push(f);
        }
    }

    let public_transit = false; // TODO
    let cost_per_road = graph.get_costs(
        starts.into_iter().collect(),
        mode,
        public_transit,
        start_time,
        start_time + limit,
    );
    let mut intersection_points: Vec<Coord> = Vec::new();
    for (r, cost) in cost_per_road {
        if !route_roads.contains(&r) {
            let road = &graph.roads[r.0];
            let mut f = Feature::from(Geometry::from(&graph.mercator.to_wgs84(&road.linestring)));
            f.set_property("kind", "buffer");
            f.set_property("cost_seconds", cost.as_secs());
            features.push(f);

            intersection_points.push(graph.intersections[road.src_i.0].point.into());
            intersection_points.push(graph.intersections[road.dst_i.0].point.into());
        }
    }

    // Build a convex hull around all the explored roads. It's only defined on polygons, so make up
    // a nonsense polygon first
    let hull = Polygon::new(LineString(intersection_points), Vec::new()).convex_hull();
    let mut f = Feature::from(Geometry::from(&graph.mercator.to_wgs84(&hull)));
    f.set_property("kind", "hull");
    features.push(f);

    let total_population = intersect_zones(graph, &mut features, MultiPolygon(vec![hull]));

    Ok(serde_json::to_string(&FeatureCollection {
        features,
        bbox: None,
        foreign_members: Some(
            serde_json::json!({
                "total_population": total_population,
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
    })?)
}

fn intersect_zones(graph: &Graph, features: &mut Vec<Feature>, hull: MultiPolygon) -> u32 {
    let mut total = 0;

    // TODO May want to prune in huge areas
    for zone in &graph.zones {
        // TODO This crashes sometimes and can't be reasonably caught in WASM
        let hit = hull.intersection(&zone.geom);
        let hit_area_km2 = 1e-6 * hit.unsigned_area();
        let pct = hit_area_km2 / zone.area_km2;
        let population = ((zone.population as f64) * pct) as u32;

        let mut f = Feature::from(Geometry::from(&graph.mercator.to_wgs84(&hit)));
        f.set_property("kind", "zone_overlap");
        f.set_property("population", population);
        f.set_property("pct", pct);
        features.push(f);

        total += population;
    }

    total
}
