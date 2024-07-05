use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufWriter;

use anyhow::Result;
use chrono::NaiveTime;
use flatgeobuf::{
    FeatureProperties, FgbFeature, FgbWriter, GeometryType, GeozeroGeometry, HttpFgbReader,
};
use geo::LineString;
use geozero::{ColumnValue, PropertyProcessor};
use serde::{Deserialize, Serialize};
use utils::Mercator;

use super::{orig_ids, GtfsModel, Route, RouteID, Stop, StopID, Trip};
use crate::graph::RoadID;

impl GtfsModel {
    pub fn to_fgb(&self, filename: &str) -> Result<()> {
        let mut fgb = FgbWriter::create("gtfs", GeometryType::LineString)?;
        // TODO Did we not need to add a json or string col?

        for (variant, linestring) in group_variants(self) {
            fgb.add_feature_geom(geo::Geometry::LineString(linestring), |feature| {
                feature
                    .property(
                        0,
                        "data",
                        &ColumnValue::String(&serde_json::to_string(&variant).unwrap()),
                    )
                    .unwrap();
            })?;
        }

        let mut out = BufWriter::new(File::create(filename)?);
        fgb.write(&mut out)?;
        println!("Wrote {filename}");
        Ok(())
    }

    pub async fn from_fgb(url: &str, mercator: &Mercator) -> Result<Self> {
        let bbox = &mercator.wgs84_bounds;
        let mut fgb = HttpFgbReader::open(url)
            .await?
            .select_bbox(bbox.min().x, bbox.min().y, bbox.max().x, bbox.max().y)
            .await?;

        let mut gtfs = GtfsModel::empty();
        while let Some(feature) = fgb.next().await? {
            // TODO Is there some serde magic?
            let geometry = get_linestring(feature)?;
            let variant: RouteVariant =
                serde_json::from_str(&feature.property::<String>("data").unwrap())?;
            info!("got {:?} from FGB", geometry);

            // Fill out the route
            let route_id = if let Some(idx) = gtfs
                .routes
                .iter()
                .position(|r| r.orig_id == variant.route.orig_id)
            {
                RouteID(idx)
            } else {
                gtfs.routes.push(variant.route);
                RouteID(gtfs.routes.len() - 1)
            };

            // Fill out the stops
            let mut stop_ids = Vec::new();
            for ((orig_stop_id, stop_name), point) in
                variant.stop_info.into_iter().zip(geometry.points())
            {
                stop_ids.push(
                    if let Some(idx) = gtfs.stops.iter().position(|s| s.orig_id == orig_stop_id) {
                        StopID(idx)
                    } else {
                        gtfs.stops.push(Stop {
                            name: stop_name,
                            orig_id: orig_stop_id,
                            point: mercator.to_mercator(&point),
                            // Will fill out later
                            road: RoadID(0),
                            next_steps: Vec::new(),
                        });
                        StopID(gtfs.stops.len() - 1)
                    },
                );
            }

            // Fill out trips
            for times in variant.trips {
                gtfs.trips.push(Trip {
                    stop_sequence: stop_ids.clone().into_iter().zip(times).collect(),
                    route: route_id,
                });
            }
        }

        // TODO Need to clip

        gtfs.precompute_next_steps();

        Ok(gtfs)
    }
}

#[derive(Serialize, Deserialize)]
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

fn get_linestring(f: &FgbFeature) -> Result<LineString> {
    let mut p = geozero::geo_types::GeoWriter::new();
    f.process_geom(&mut p)?;
    match p.take_geometry().unwrap() {
        geo::Geometry::LineString(ls) => Ok(ls),
        _ => bail!("Wrong type in fgb"),
    }
}
