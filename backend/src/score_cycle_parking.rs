use std::collections::HashSet;
use std::time::Duration;

use anyhow::Result;
use chrono::NaiveTime;

use crate::graph::{Graph, Mode};

// Return GeoJSON points for each POI, with a score to the nearest cycle parking
pub fn calculate(graph: &Graph) -> Result<String> {
    let poi_kinds = ["cafe", "pub", "restaurant", "bank", "nightclub"];
    let limit = Duration::from_secs(10 * 60);
    // Exact time doesn't matter
    let start_time = NaiveTime::from_hms_opt(7, 0, 0).unwrap();
    let end_time = start_time + limit;

    let mut cycle_parking_roads = HashSet::new();
    for road in &graph.roads {
        if road.amenities[Mode::Foot]
            .iter()
            .any(|a| graph.amenities[a.0].kind == "bicycle_parking")
        {
            cycle_parking_roads.insert(road.id);
        }
    }

    let mut features = Vec::new();
    for amenity in &graph.amenities {
        if !poi_kinds.contains(&amenity.kind.as_str()) {
            continue;
        }

        let costs = crate::isochrone::get_costs(
            graph,
            amenity.point.into(),
            Mode::Foot,
            false,
            start_time,
            end_time,
        );
        if let Some(cost) = costs
            .into_iter()
            .filter(|(r, _)| cycle_parking_roads.contains(r))
            .map(|(_, cost)| cost)
            .min()
        {
            let mut f = geojson::Feature::from(geojson::Geometry::from(
                &graph.mercator.to_wgs84(&amenity.point),
            ));
            f.set_property("cost", cost.as_secs());
            features.push(f);
        }
    }
    let gj = geojson::GeoJson::from(features);
    Ok(serde_json::to_string(&gj)?)
}
