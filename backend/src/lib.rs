//#[macro_use]
//extern crate log;

use std::sync::Once;

use geo::Coord;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

use graph::{Graph, Mode};

mod graph;
mod isochrone;
mod scrape;

static START: Once = Once::new();

// TODO Rename
#[wasm_bindgen]
pub struct MapModel {
    graph: Graph,
}

#[wasm_bindgen]
impl MapModel {
    /// Call with bytes of an osm.pbf or osm.xml string
    #[wasm_bindgen(constructor)]
    pub fn new(input_bytes: &[u8]) -> Result<MapModel, JsValue> {
        // Panics shouldn't happen, but if they do, console.log them.
        console_error_panic_hook::set_once();
        START.call_once(|| {
            console_log::init_with_level(log::Level::Info).unwrap();
        });

        Ok(MapModel {
            graph: Graph::new(input_bytes).map_err(err_to_js)?,
        })
    }

    /// Returns a GeoJSON string. Just shows the full network
    #[wasm_bindgen()]
    pub fn render(&self) -> Result<String, JsValue> {
        self.graph.render().map_err(err_to_js)
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
            // TODO error plumbing
            x => panic!("bad input {x}"),
        };
        isochrone::calculate(&self.graph, start, mode).map_err(err_to_js)
    }
}

#[derive(Deserialize)]
pub struct IsochroneRequest {
    x: f64,
    y: f64,
    mode: String,
}

fn err_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}
