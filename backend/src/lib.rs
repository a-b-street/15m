#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use std::collections::HashSet;
use std::sync::Once;
use std::time::Duration;

use chrono::NaiveTime;
use geo::{Coord, LineString};
use geojson::{de::deserialize_geometry, Feature, GeoJson, Geometry};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

pub use graph::{Graph, GtfsSource, Mode};
pub use gtfs::GtfsModel;
pub use timer::Timer;

mod buffer;
mod graph;
mod gtfs;
mod isochrone;
mod score;
mod timer;

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
        population_url: Option<String>,
        progress_cb: Option<js_sys::Function>,
    ) -> Result<MapModel, JsValue> {
        // Panics shouldn't happen, but if they do, console.log them.
        console_error_panic_hook::set_once();
        START.call_once(|| {
            console_log::init_with_level(log::Level::Info).unwrap();
        });

        let gtfs = match gtfs_url {
            Some(url) => graph::GtfsSource::Geomedea(url),
            None => graph::GtfsSource::None,
        };
        let graph = if is_osm {
            Graph::new(
                input_bytes,
                gtfs,
                population_url,
                Timer::new("build graph", progress_cb),
            )
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

    #[wasm_bindgen(js_name = renderZones)]
    pub fn render_zones(&self) -> Result<String, JsValue> {
        self.graph.render_zones().map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = isochrone)]
    pub fn isochrone(&self, input: JsValue) -> Result<String, JsValue> {
        let req: IsochroneRequest = serde_wasm_bindgen::from_value(input)?;
        let start = self
            .graph
            .mercator
            .pt_to_mercator(Coord { x: req.x, y: req.y });
        let mode = Mode::parse(&req.mode).map_err(err_to_js)?;
        isochrone::calculate(
            &self.graph,
            start,
            mode,
            // TODO Hack
            serde_json::from_str(&format!("\"{}\"", req.style)).map_err(err_to_js)?,
            req.mode == "transit",
            NaiveTime::parse_from_str(&req.start_time, "%H:%M").map_err(err_to_js)?,
            Duration::from_secs(req.max_seconds),
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

        let mode = Mode::parse(&req.mode).map_err(err_to_js)?;
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

        let start_time = NaiveTime::parse_from_str(&req.start_time, "%H:%M").map_err(err_to_js)?;
        let limit = Duration::from_secs(req.max_seconds);

        crate::buffer::buffer_route(&self.graph, mode, steps, start_time, limit).map_err(err_to_js)
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

    #[wasm_bindgen(js_name = snapRoute)]
    pub fn snap_route(&self, input: JsValue) -> Result<String, JsValue> {
        let req: SnapRouteRequest = serde_wasm_bindgen::from_value(input)?;
        let mode = Mode::parse(&req.mode).map_err(err_to_js)?;
        let mut linestrings = Vec::new();
        for mut input in
            geojson::de::deserialize_feature_collection_str_to_vec::<GeoJsonLineString>(&req.input)
                .map_err(err_to_js)?
        {
            self.graph
                .mercator
                .to_mercator_in_place(&mut input.geometry);
            linestrings.push(input.geometry);
        }

        let mut output = Vec::new();
        for input in linestrings {
            let (_, snapped) = self.graph.snap_route(&input, mode).map_err(err_to_js)?;
            output.push(Feature::from(Geometry::from(
                &self.graph.mercator.to_wgs84(&snapped),
            )));
        }
        Ok(serde_json::to_string(&GeoJson::from(output)).map_err(err_to_js)?)
    }
}

// Non WASM methods
// TODO Reconsider these. Benchmark should use Graph. MapModel should just be a thin WASM layer.
impl MapModel {
    pub fn from_graph_bytes(input_bytes: &[u8]) -> Result<MapModel, JsValue> {
        let graph = bincode::deserialize_from(input_bytes).map_err(err_to_js)?;
        Ok(MapModel { graph })
    }

    pub fn route_from_req(&self, req: &RouteRequest) -> Result<String, JsValue> {
        let mode = Mode::parse(&req.mode).map_err(err_to_js)?;
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
            self.graph
                .transit_route_gj(
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
    // TODO Rename lon, lat to be clear?
    x: f64,
    y: f64,
    mode: String,
    style: String,
    start_time: String,
    max_seconds: u64,
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

#[derive(Deserialize)]
struct SnapRouteRequest {
    input: String,
    mode: String,
}

#[derive(Deserialize)]
struct GeoJsonLineString {
    #[serde(deserialize_with = "deserialize_geometry")]
    geometry: LineString,
}

fn err_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}
