#[macro_use]
extern crate anyhow;

use std::collections::HashSet;
use std::sync::Once;
use std::time::Duration;

use chrono::NaiveTime;
use geo::{Coord, LineString};
use geojson::{de::deserialize_geometry, Feature, GeoJson, Geometry};
use graph::{Graph, Mode, Timer};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::amenity::Amenities;
use crate::zone::Zones;

mod amenity;
mod buffer;
mod isochrone;
mod score;
mod zone;

static START: Once = Once::new();

// TODO Rename
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct MapModel {
    graph: Graph,
    zones: Zones,
    amenities: Amenities,
}

#[wasm_bindgen]
impl MapModel {
    #[wasm_bindgen(constructor)]
    pub async fn new(
        input_bytes: &[u8],
        gtfs_url: Option<String>,
        population_url: Option<String>,
        progress_cb: Option<js_sys::Function>,
    ) -> Result<MapModel, JsValue> {
        // Panics shouldn't happen, but if they do, console.log them.
        console_error_panic_hook::set_once();
        START.call_once(|| {
            console_log::init_with_level(log::Level::Info).unwrap();
        });

        let mut timer = Timer::new("build graph", progress_cb);
        let model = MapModel::create(input_bytes, gtfs_url, population_url, &mut timer)
            .await
            .map_err(err_to_js)?;
        timer.done();

        Ok(model)
    }

    #[wasm_bindgen(js_name = loadFile)]
    pub fn load_file(input_bytes: &[u8]) -> Result<MapModel, JsValue> {
        bincode::deserialize_from(input_bytes).map_err(err_to_js)
    }

    /// Returns a GeoJSON string. Just shows the full network
    #[wasm_bindgen(js_name = renderDebug)]
    pub fn render_debug(&self) -> Result<String, JsValue> {
        let mut fc = self.graph.render_debug();
        for a in &self.amenities.amenities {
            fc.features.push(a.to_gj(&self.graph.mercator));
        }
        serde_json::to_string(&fc).map_err(err_to_js)
    }

    /// Returns a GeoJSON string showing all amenities
    #[wasm_bindgen(js_name = renderAmenities)]
    pub fn render_amenities(&self) -> Result<String, JsValue> {
        self.amenities
            .render_amenities(&self.graph.mercator)
            .map_err(err_to_js)
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
        self.zones
            .render_zones(&self.graph.mercator)
            .map_err(err_to_js)
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
            &self.amenities,
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

        let route = if req.mode == "transit" {
            todo!()
        } else {
            self.graph.router[mode]
                .route(&self.graph, start, end)
                .map_err(err_to_js)?
        };

        let start_time = NaiveTime::parse_from_str(&req.start_time, "%H:%M").map_err(err_to_js)?;
        let limit = Duration::from_secs(req.max_seconds);

        crate::buffer::buffer_route(
            &self.graph,
            &self.zones,
            mode,
            vec![route],
            start_time,
            limit,
        )
        .map_err(err_to_js)
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
            &self.amenities,
            poi_kinds,
            limit,
            Timer::new("score", progress_cb),
        )
        .map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = snapAndBufferRoute)]
    pub fn snap_and_buffer_route(&self, input: JsValue) -> Result<String, JsValue> {
        let req: SnapRouteRequest = serde_wasm_bindgen::from_value(input)?;
        let mode = Mode::parse(&req.mode).map_err(err_to_js)?;

        let mut routes = Vec::new();
        for mut input in
            geojson::de::deserialize_feature_collection_str_to_vec::<GeoJsonLineString>(&req.input)
                .map_err(err_to_js)?
        {
            self.graph
                .mercator
                .to_mercator_in_place(&mut input.geometry);
            routes.push(
                self.graph
                    .snap_route(&input.geometry, mode)
                    .map_err(err_to_js)?,
            );
        }

        let start_time = NaiveTime::parse_from_str(&req.start_time, "%H:%M").map_err(err_to_js)?;
        let limit = Duration::from_secs(req.max_seconds);

        crate::buffer::buffer_route(&self.graph, &self.zones, mode, routes, start_time, limit)
            .map_err(err_to_js)
    }
}

// Non WASM methods, also used by the CLI
impl MapModel {
    pub async fn create(
        input_bytes: &[u8],
        gtfs_url: Option<String>,
        population_url: Option<String>,
        timer: &mut Timer,
    ) -> anyhow::Result<MapModel> {
        let mut amenities = Amenities::new();
        let modify_roads = |_roads: &mut Vec<graph::Road>| {};
        let mut graph = Graph::new(input_bytes, &mut amenities, modify_roads, timer)?;

        graph
            .setup_gtfs(
                match gtfs_url {
                    Some(url) => graph::GtfsSource::Geomedea(url),
                    None => graph::GtfsSource::None,
                },
                timer,
            )
            .await?;
        amenities.finalize(&graph, timer);
        let zones = Zones::load(population_url, &graph.mercator, timer).await?;

        Ok(MapModel {
            graph,
            zones,
            amenities,
        })
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
            let linestring = self.graph.router[mode]
                .route(&self.graph, start, end)
                .map_err(err_to_js)?
                .linestring(&self.graph);
            let mut f = Feature::from(Geometry::from(&self.graph.mercator.to_wgs84(&linestring)));
            f.set_property("kind", "road");
            Ok(serde_json::to_string(&GeoJson::from(vec![f])).map_err(err_to_js)?)
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
    start_time: String,
    max_seconds: u64,
}

#[derive(Deserialize)]
struct GeoJsonLineString {
    #[serde(deserialize_with = "deserialize_geometry")]
    geometry: LineString,
}

fn err_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}
