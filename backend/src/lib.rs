#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use std::sync::Once;

use geo::Coord;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

pub use graph::{Graph, Mode};
pub use timer::Timer;

mod amenity;
mod costs;
mod graph;
mod gtfs;
mod isochrone;
mod route;
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
    pub fn new(
        input_bytes: &[u8],
        is_osm: bool,
        progress_cb: Option<js_sys::Function>,
    ) -> Result<MapModel, JsValue> {
        // Panics shouldn't happen, but if they do, console.log them.
        console_error_panic_hook::set_once();
        START.call_once(|| {
            console_log::init_with_level(log::Level::Info).unwrap();
        });

        let graph = if is_osm {
            Graph::new(input_bytes, None, Timer::new("build graph", progress_cb))
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

    /// Return a polygon covering the world, minus a hole for the boundary, in WGS84
    #[wasm_bindgen(js_name = getInvertedBoundary)]
    pub fn get_inverted_boundary(&self) -> Result<String, JsValue> {
        self.graph.get_inverted_boundary().map_err(err_to_js)
    }

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
            // TODO Unimplemented
            "transit" => Mode::Foot,
            // TODO error plumbing
            x => panic!("bad input {x}"),
        };
        isochrone::calculate(
            &self.graph,
            start,
            mode,
            req.contours,
            Timer::new("isochrone request", None),
        )
        .map_err(err_to_js)
    }

    // mut because of path_calc
    #[wasm_bindgen(js_name = route)]
    pub fn route(&self, input: JsValue) -> Result<String, JsValue> {
        let req: RouteRequest = serde_wasm_bindgen::from_value(input)?;
        let mode = match req.mode.as_str() {
            "car" => Mode::Car,
            "bicycle" => Mode::Bicycle,
            "foot" => Mode::Foot,
            // For endpoint matching only
            "transit" => Mode::Foot,
            // TODO error plumbing
            x => panic!("bad input {x}"),
        };
        let start = self.graph.closest_intersection[mode]
            .nearest_neighbor(&x_y(self.graph.mercator.pt_to_mercator(Coord {
                x: req.x1,
                y: req.y1,
            })))
            .unwrap()
            .data;
        let end = self.graph.closest_intersection[mode]
            .nearest_neighbor(&x_y(self.graph.mercator.pt_to_mercator(Coord {
                x: req.x2,
                y: req.y2,
            })))
            .unwrap()
            .data;

        if req.mode == "transit" {
            transit_route::route(
                &self.graph,
                start,
                end,
                req.debug_search,
                req.use_heuristic,
                Timer::new("route request", None),
            )
            .map_err(err_to_js)
        } else {
            self.graph.router[mode]
                .route(&self.graph, start, end)
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
}

#[derive(Deserialize)]
pub struct RouteRequest {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    mode: String,
    // TODO Only works for transit
    debug_search: bool,
    use_heuristic: bool,
}

fn err_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}

fn x_y(c: Coord) -> [f64; 2] {
    [c.x, c.y]
}
