use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufWriter;

use anyhow::Result;
use chrono::NaiveTime;
use flatgeobuf::geozero::geojson::GeoJsonString;
use flatgeobuf::{FgbWriter, GeometryType};
use geo::LineString;
use geojson::{Feature, Geometry};
use serde::Serialize;

use super::{orig_ids, GtfsModel, Route, StopID};

impl GtfsModel {
    pub fn to_fgb(&self, filename: &str) -> Result<()> {
        let mut fgb = FgbWriter::create("gtfs", GeometryType::LineString)?;
        // TODO json col

        for (variant, linestring) in group_variants(self) {
            // TODO Is there a way to avoid this round-trip?
            let mut f = Feature::from(Geometry::from(&linestring));
            f.set_property("data", serde_json::to_value(&variant)?);
            fgb.add_feature(GeoJsonString(serde_json::to_string(&f)?))?;
        }

        let mut out = BufWriter::new(File::create(filename)?);
        fgb.write(&mut out)?;
        println!("Wrote {filename}");
        Ok(())
    }

    // TODO Will need to clip to a map later
}

#[derive(Serialize)]
struct RouteVariant {
    // Per stop, (original ID and name)
    pub stop_info: Vec<(orig_ids::StopID, String)>,

    // Each one has an arrival time per stop
    pub trips: Vec<Vec<NaiveTime>>,

    // Metadata
    pub route: Route,
}

// In GTFS, routes contain many trips, each with a stop sequence. But there are really "variants"
// of stop sequences. We need to group by those.
fn group_variants(gtfs: &GtfsModel) -> Vec<(RouteVariant, LineString)> {
    let mut variants: BTreeMap<Vec<StopID>, (RouteVariant, LineString)> = BTreeMap::new();

    for trip in &gtfs.trips {
        let stop_sequence: Vec<StopID> = trip.stop_sequence.iter().map(|(s, _)| *s).collect();
        let trip_times = trip.stop_sequence.iter().map(|(_, t)| *t).collect();

        variants
            .entry(stop_sequence.clone())
            .or_insert_with(|| {
                let mut stop_info = Vec::new();
                let mut pts = Vec::new();
                for s in &stop_sequence {
                    let stop = &gtfs.stops[s.0];
                    stop_info.push((stop.orig_id.clone(), stop.name.clone()));
                    pts.push(stop.point.into());
                }

                (
                    RouteVariant {
                        stop_info,
                        trips: Vec::new(),
                        route: gtfs.routes[trip.route.0].clone(),
                    },
                    LineString::new(pts),
                )
            })
            .0
            .trips
            .push(trip_times);
    }

    variants.into_values().collect()
}
