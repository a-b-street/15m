#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use std::collections::HashSet;
use std::sync::Once;
use std::time::Duration;

use chrono::NaiveTime;
use geo::Coord;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

pub use graph::{Graph, Mode};
pub use gtfs::GtfsModel;
pub use timer::Timer;

mod amenity;
mod costs;
mod graph;
mod gtfs;
mod isochrone;
mod route;
mod score;
mod scrape;
mod timer;
mod transit_route;

static START: Once = Once::new();

// TODO Rename
#[wasm_bindgen]
pub struct MapModel {
    graph: Graph,
}

#[wasm_bindgen]
impl MapModel {
    /// If is_osm is true, expect bytes of an osm.pbf or osm.xml string. Otherwise, expect a
    /// bincoded graph
    #[wasm_bindgen(constructor)]
    pub async fn new(
        input_bytes: &[u8],
        is_osm: bool,
        gtfs_url: Option<String>,
        progress_cb: Option<js_sys::Function>,
    ) -> Result<MapModel, JsValue> {
        // Panics shouldn't happen, but if they do, console.log them.
        console_error_panic_hook::set_once();
        START.call_once(|| {
            // TODO Debugging geomedea
            console_log::init_with_level(log::Level::Debug).unwrap();
        });

        let gtfs = match gtfs_url {
            Some(url) => GtfsSource::Geomedea(url),
            None => GtfsSource::None,
        };
        let graph = if is_osm {
            Graph::new(input_bytes, gtfs, Timer::new("build graph", progress_cb))
                .await
                .map_err(err_to_js)?
        } else {
            bincode::deserialize_from(input_bytes).map_err(err_to_js)?
        };
        Ok(MapModel { graph })
    }

    /// Returns a GeoJSON string. Just shows the full network
    #[wasm_bindgen(js_name = renderDebug)]
    pub fn render_debug(&self) -> Result<String, JsValue> {
        self.graph.render_debug().map_err(err_to_js)
    }

    /// Returns a GeoJSON string showing all amenities
    #[wasm_bindgen(js_name = renderAmenities)]
    pub fn render_amenities(&self) -> Result<String, JsValue> {
        self.graph.render_amenities().map_err(err_to_js)
    }

    /// Return a polygon covering the world, minus a hole for the boundary, in WGS84
    #[wasm_bindgen(js_name = getInvertedBoundary)]
    pub fn get_inverted_boundary(&self) -> Result<String, JsValue> {
        self.graph.get_inverted_boundary().map_err(err_to_js)
    }

    /// WGS84
    #[wasm_bindgen(js_name = getBounds)]
    pub fn get_bounds(&self) -> Vec<f64> {
        let b = &self.graph.mercator.wgs84_bounds;
        vec![b.min().x, b.min().y, b.max().x, b.max().y]
    }

    #[wasm_bindgen(js_name = isochrone)]
    pub fn isochrone(&self, input: JsValue) -> Result<String, JsValue> {
        let req: IsochroneRequest = serde_wasm_bindgen::from_value(input)?;
        let start = self
            .graph
            .mercator
            .pt_to_mercator(Coord { x: req.x, y: req.y });
        let mode = match req.mode.as_str() {
            "car" => Mode::Car,
            "bicycle" => Mode::Bicycle,
            "foot" => Mode::Foot,
            // Plumbed separately
            "transit" => Mode::Foot,
            // TODO error plumbing
            x => panic!("bad input {x}"),
        };
        isochrone::calculate(
            &self.graph,
            start,
            mode,
            req.contours,
            req.mode == "transit",
            NaiveTime::parse_from_str(&req.start_time, "%H:%M").map_err(err_to_js)?,
            Timer::new("isochrone request", None),
        )
        .map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = route)]
    pub fn route(&self, input: JsValue) -> Result<String, JsValue> {
        let req: RouteRequest = serde_wasm_bindgen::from_value(input)?;
        self.route_from_req(&req)
    }

    #[wasm_bindgen(js_name = bufferRoute)]
    pub fn buffer_route(&self, input: JsValue) -> Result<String, JsValue> {
        let req: BufferRouteRequest = serde_wasm_bindgen::from_value(input)?;

        // TODO Duplicating some route_from_req boilerplate
        let mode = match req.mode.as_str() {
            "car" => Mode::Car,
            "bicycle" => Mode::Bicycle,
            "foot" => Mode::Foot,
            // For endpoint matching only
            "transit" => Mode::Foot,
            // TODO error plumbing
            x => panic!("bad input {x}"),
        };
        let start = self.graph.snap_to_road(
            self.graph.mercator.pt_to_mercator(Coord {
                x: req.x1,
                y: req.y1,
            }),
            mode,
        );
        let end = self.graph.snap_to_road(
            self.graph.mercator.pt_to_mercator(Coord {
                x: req.x2,
                y: req.y2,
            }),
            mode,
        );

        let steps = if req.mode == "transit" {
            todo!()
        } else {
            self.graph.router[mode]
                .route_steps(&self.graph, start, end)
                .map_err(err_to_js)?
        };

        let mut features = Vec::new();
        let mut route_roads = HashSet::new();
        let mut starts = HashSet::new();
        for step in steps {
            if let crate::route::PathStep::Road { road, .. } = step {
                route_roads.insert(road);
                let road = &self.graph.roads[road.0];
                starts.insert(road.src_i);
                starts.insert(road.dst_i);

                // TODO Doesn't handle the exact start/end
                let mut f = geojson::Feature::from(geojson::Geometry::from(
                    &self.graph.mercator.to_wgs84(&road.linestring),
                ));
                f.set_property("kind", "route");
                features.push(f);
            }
        }

        let public_transit = false; // TODO
        let start_time = NaiveTime::parse_from_str(&req.start_time, "%H:%M").map_err(err_to_js)?;
        let limit = Duration::from_secs(req.max_seconds);
        let cost_per_road = self.graph.get_costs(
            starts.into_iter().collect(),
            mode,
            public_transit,
            start_time,
            start_time + limit,
        );
        for (r, cost) in cost_per_road {
            if !route_roads.contains(&r) {
                let mut f = geojson::Feature::from(geojson::Geometry::from(
                    &self
                        .graph
                        .mercator
                        .to_wgs84(&self.graph.roads[r.0].linestring),
                ));
                f.set_property("kind", "buffer");
                f.set_property("cost_seconds", cost.as_secs());
                features.push(f);
            }
        }

        let gj = geojson::GeoJson::from(features);
        serde_json::to_string(&gj).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = score)]
    pub fn score(
        &self,
        input: JsValue,
        progress_cb: Option<js_sys::Function>,
    ) -> Result<String, JsValue> {
        let req: ScoreRequest = serde_wasm_bindgen::from_value(input)?;
        let poi_kinds: HashSet<String> = req.poi_kinds.into_iter().collect();
        let limit = Duration::from_secs(req.max_seconds);
        score::calculate(
            &self.graph,
            poi_kinds,
            limit,
            Timer::new("score", progress_cb),
        )
        .map_err(err_to_js)
    }
}

// Non WASM methods
impl MapModel {
    pub fn from_graph_bytes(input_bytes: &[u8]) -> Result<MapModel, JsValue> {
        let graph = bincode::deserialize_from(input_bytes).map_err(err_to_js)?;
        Ok(MapModel { graph })
    }

    pub fn route_from_req(&self, req: &RouteRequest) -> Result<String, JsValue> {
        let mode = match req.mode.as_str() {
            "car" => Mode::Car,
            "bicycle" => Mode::Bicycle,
            "foot" => Mode::Foot,
            // For endpoint matching only
            "transit" => Mode::Foot,
            // TODO error plumbing
            x => panic!("bad input {x}"),
        };
        let start = self.graph.snap_to_road(
            self.graph.mercator.pt_to_mercator(Coord {
                x: req.x1,
                y: req.y1,
            }),
            mode,
        );
        let end = self.graph.snap_to_road(
            self.graph.mercator.pt_to_mercator(Coord {
                x: req.x2,
                y: req.y2,
            }),
            mode,
        );

        if req.mode == "transit" {
            transit_route::route_gj(
                &self.graph,
                start,
                end,
                req.debug_search,
                req.use_heuristic,
                NaiveTime::parse_from_str(&req.start_time, "%H:%M").map_err(err_to_js)?,
                Timer::new("route request", None),
            )
            .map_err(err_to_js)
        } else {
            self.graph.router[mode]
                .route_gj(&self.graph, start, end)
                .map_err(err_to_js)
        }
    }
}

#[derive(Deserialize)]
pub struct IsochroneRequest {
    x: f64,
    y: f64,
    mode: String,
    contours: bool,
    start_time: String,
}

#[derive(Deserialize)]
pub struct RouteRequest {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub mode: String,
    // TODO Only works for transit
    pub debug_search: bool,
    pub use_heuristic: bool,
    pub start_time: String,
}

#[derive(Deserialize)]
pub struct BufferRouteRequest {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub mode: String,
    pub use_heuristic: bool,
    pub start_time: String,
    pub max_seconds: u64,
}

#[derive(Deserialize)]
pub struct ScoreRequest {
    poi_kinds: Vec<String>,
    max_seconds: u64,
}

fn err_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}

pub enum GtfsSource {
    Dir(String),
    Geomedea(String),
    None,
}
