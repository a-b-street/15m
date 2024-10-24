use std::collections::{HashMap, HashSet};
use std::time::Duration;

use anyhow::Result;
use chrono::NaiveTime;
use graph::{Graph, ProfileID, RoadID, Timer};

use crate::Amenities;

// Return GeoJSON points for each POI, with info about that POI, a score to the nearest cycle
// parking, and the location of that parking
pub fn calculate(
    graph: &Graph,
    amenities: &Amenities,
    profile: ProfileID,
    poi_kinds: HashSet<String>,
    limit: Duration,
    mut timer: Timer,
) -> Result<String> {
    // Exact time doesn't matter
    let start_time = NaiveTime::from_hms_opt(7, 0, 0).unwrap();
    let end_time = start_time + limit;

    // Per road, store one arbitrary point of parking
    timer.step("look for targets (cycle parking)");
    let mut cycle_parking_roads = HashMap::new();
    for (idx, list) in amenities.per_road.iter().enumerate() {
        if let Some(a) = list[profile.0]
            .iter()
            .find(|a| amenities.amenities[a.0].kind == "bicycle_parking")
        {
            cycle_parking_roads.insert(RoadID(idx), amenities.amenities[a.0].point);
        }
    }

    timer.step(format!(
        "calculate for amenities (up to {})",
        amenities.amenities.len()
    ));
    let mut features = Vec::new();
    for amenity in &amenities.amenities {
        if !poi_kinds.contains(&amenity.kind) {
            continue;
        }

        let costs = graph.get_costs(
            vec![
                graph
                    .snap_to_road(amenity.point.into(), profile)
                    .intersection,
            ],
            profile,
            false,
            start_time,
            end_time,
        );
        if let Some((r, cost)) = costs
            .into_iter()
            .filter(|(r, _)| cycle_parking_roads.contains_key(r))
            .min_by_key(|(_, cost)| *cost)
        {
            let mut f = geojson::Feature::from(geojson::Geometry::from(
                &graph.mercator.to_wgs84(&amenity.point),
            ));
            f.set_property(
                "poi",
                format!(
                    "{} ({})",
                    amenity
                        .name
                        .clone()
                        .unwrap_or_else(|| "unnamed".to_string()),
                    amenity.kind
                ),
            );
            f.set_property("cost", cost.as_secs());
            let pt = graph.mercator.to_wgs84(&cycle_parking_roads[&r]);
            f.set_property("closest_lon", pt.x());
            f.set_property("closest_lat", pt.y());
            features.push(f);
        }
    }
    let gj = geojson::GeoJson::from(features);
    let out = serde_json::to_string(&gj)?;
    timer.done();
    Ok(out)
}
