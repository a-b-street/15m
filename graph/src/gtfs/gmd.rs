use anyhow::{bail, Result};
use chrono::NaiveTime;
use futures_util::StreamExt;
use geo::{Contains, Coord, LineString};
use geomedea::{Bounds, Geometry, LngLat, Properties, PropertyValue};
use utils::Mercator;

use super::{orig_ids, GtfsModel, Route, RouteID, Stop, StopID, Trip};
use crate::RoadID;

impl GtfsModel {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn to_geomedea(&self, filename: &str) -> Result<()> {
        use std::fs::File;
        use std::io::BufWriter;

        use geomedea::{Feature, Geometry, Writer};

        let out = BufWriter::new(File::create(filename)?);
        let mut writer = Writer::new(out, true)?;

        for (variant, linestring) in group_variants(self) {
            // TODO geozero to this translation better?
            let geom = Geometry::LineString(geomedea::LineString::new(
                linestring
                    .0
                    .into_iter()
                    .map(|pt| geomedea::LngLat::degrees(pt.x, pt.y))
                    .collect(),
            ));
            let props = variant.encode()?;
            writer.add_feature(&Feature::new(geom, props))?;
        }

        writer.finish()?;
        println!("Wrote {filename}");
        Ok(())
    }

    pub async fn from_geomedea(url: &str, mercator: &Mercator) -> Result<Self> {
        let bbox = &mercator.wgs84_bounds;
        let mut reader = geomedea::HttpReader::open(url).await?;
        let mut feature_stream = reader
            .select_bbox(&Bounds::from_corners(
                &LngLat::degrees(bbox.min().x, bbox.min().y),
                &LngLat::degrees(bbox.max().x, bbox.max().y),
            ))
            .await?;

        let mut gtfs = GtfsModel::empty();
        while let Some(feature) = feature_stream.next().await {
            let feature = feature?;
            let (geometry, properties) = feature.into_inner();

            let variant = RouteVariant::decode(properties)?;

            let linestring = match geometry {
                Geometry::LineString(ls) => LineString::new(
                    ls.points()
                        .into_iter()
                        .map(|pt| Coord {
                            x: pt.lng_degrees(),
                            y: pt.lat_degrees(),
                        })
                        .collect(),
                ),
                _ => bail!("Wrong Geometry type"),
            };

            // Fill out the stops
            let mut stop_ids = Vec::new();
            // Have a true/false for each entry in the full stop_sequence
            let mut keep_stops = Vec::new();
            for ((orig_stop_id, stop_name), point) in
                variant.stop_info.into_iter().zip(linestring.points())
            {
                // Mimic what scrape.rs does, removing stops outside the bounding box.
                // TODO Be even more precise -- inside the polygon
                if !mercator.wgs84_bounds.contains(&point) {
                    keep_stops.push(false);
                    continue;
                }
                keep_stops.push(true);

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

            // If all stops were out of bounds, we got something totally irrelevant
            if stop_ids.is_empty() {
                continue;
            }

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

            // Fill out trips
            for times in variant.trips {
                // We might've clipped out some stops
                let clipped_times = times
                    .into_iter()
                    .zip(&keep_stops)
                    .filter(|(_, ok)| **ok)
                    .map(|(t, _)| t)
                    .collect::<Vec<_>>();
                assert_eq!(stop_ids.len(), clipped_times.len());

                gtfs.trips.push(Trip {
                    stop_sequence: stop_ids.clone().into_iter().zip(clipped_times).collect(),
                    route: route_id,
                });
            }
        }

        gtfs.precompute_next_steps();

        Ok(gtfs)
    }
}

struct RouteVariant {
    // Per stop, (original ID and name)
    pub stop_info: Vec<(orig_ids::StopID, String)>,

    // Each one has an arrival time per stop
    pub trips: Vec<Vec<NaiveTime>>,

    // Metadata
    pub route: Route,
}

impl RouteVariant {
    #[cfg(not(target_arch = "wasm32"))]
    fn encode(&self) -> Result<Properties> {
        use chrono::Timelike;

        let mut props = Properties::empty();

        // These two fields aren't usually too big, so just use JSON
        props.insert(
            "stop_info".to_string(),
            PropertyValue::Bytes(serde_json::to_vec(&self.stop_info)?),
        );
        props.insert(
            "route".to_string(),
            PropertyValue::Bytes(serde_json::to_vec(&self.route)?),
        );

        // NaiveTime's serde encodes as strings by default! For GTFS arrival times, we don't even
        // care about subsecond precision.
        // TODO Some kind of delta encoding here could probably be useful
        props.insert(
            "trips".to_string(),
            PropertyValue::Vec(
                self.trips
                    .iter()
                    .map(|times| {
                        PropertyValue::Vec(
                            times
                                .iter()
                                .map(|t| PropertyValue::UInt32(t.num_seconds_from_midnight()))
                                .collect(),
                        )
                    })
                    .collect(),
            ),
        );

        Ok(props)
    }

    fn decode(props: Properties) -> Result<Self> {
        let stop_info = match props.get("stop_info") {
            Some(PropertyValue::Bytes(bytes)) => serde_json::from_slice(bytes)?,
            _ => bail!("stop_info missing or wrong type"),
        };
        let route = match props.get("route") {
            Some(PropertyValue::Bytes(bytes)) => serde_json::from_slice(bytes)?,
            _ => bail!("route missing or wrong type"),
        };

        let mut trips = Vec::new();
        let Some(PropertyValue::Vec(raw_trips)) = props.get("trips") else {
            bail!("trips missing or wrong type");
        };
        for trip in raw_trips {
            let mut times = Vec::new();
            let PropertyValue::Vec(raw_times) = trip else {
                bail!("wrong inner type inside trips");
            };
            for t in raw_times {
                let PropertyValue::UInt32(seconds) = t else {
                    bail!("wrong inner type inside trips");
                };
                times.push(NaiveTime::from_num_seconds_from_midnight_opt(*seconds, 0).unwrap());
            }
            trips.push(times);
        }

        Ok(Self {
            stop_info,
            route,
            trips,
        })
    }
}

// In GTFS, routes contain many trips, each with a stop sequence. But there are really "variants"
// of stop sequences. We need to group by those.
#[cfg(not(target_arch = "wasm32"))]
fn group_variants(gtfs: &GtfsModel) -> Vec<(RouteVariant, LineString)> {
    use std::collections::BTreeMap;

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
