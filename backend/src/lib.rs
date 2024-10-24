#[macro_use]
extern crate anyhow;

use std::collections::HashSet;
use std::sync::Once;
use std::time::Duration;

use chrono::NaiveTime;
use geo::{Coord, LineString};
use geojson::{de::deserialize_geometry, Feature, GeoJson, Geometry};
use graph::{Graph, ProfileID, Timer};
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
        // Panics shouldn't happen, but if they do, console.log them.
        console_error_panic_hook::set_once();
        START.call_once(|| {
            console_log::init_with_level(log::Level::Info).unwrap();
        });

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
        let profile = self.parse_profile(&req.profile)?;
        isochrone::calculate(
            &self.graph,
            &self.amenities,
            start,
            profile,
            // TODO Hack
            serde_json::from_str(&format!("\"{}\"", req.style)).map_err(err_to_js)?,
            req.transit,
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

        let profile = self.parse_profile(&req.profile)?;
        let start = self.graph.snap_to_road(
            self.graph.mercator.pt_to_mercator(Coord {
                x: req.x1,
                y: req.y1,
            }),
            profile,
        );
        let end = self.graph.snap_to_road(
            self.graph.mercator.pt_to_mercator(Coord {
                x: req.x2,
                y: req.y2,
            }),
            profile,
        );

        let route = if req.transit {
            todo!()
        } else {
            self.graph.routers[profile.0]
                .route(&self.graph, start, end)
                .map_err(err_to_js)?
        };

        let start_time = NaiveTime::parse_from_str(&req.start_time, "%H:%M").map_err(err_to_js)?;
        let limit = Duration::from_secs(req.max_seconds);

        self.buffer_routes(vec![route], profile, start_time, limit)
            .map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = score)]
    pub fn score(
        &self,
        input: JsValue,
        progress_cb: Option<js_sys::Function>,
    ) -> Result<String, JsValue> {
        let req: ScoreRequest = serde_wasm_bindgen::from_value(input)?;
        let profile = self.parse_profile(&req.profile)?;
        let poi_kinds: HashSet<String> = req.poi_kinds.into_iter().collect();
        let limit = Duration::from_secs(req.max_seconds);
        score::calculate(
            &self.graph,
            &self.amenities,
            profile,
            poi_kinds,
            limit,
            Timer::new("score", progress_cb),
        )
        .map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = snapAndBufferRoute)]
    pub fn snap_and_buffer_route(
        &self,
        input: JsValue,
        progress_cb: Option<js_sys::Function>,
    ) -> Result<String, JsValue> {
        let req: SnapRouteRequest = serde_wasm_bindgen::from_value(input)?;
        let profile = self.parse_profile(&req.profile)?;
        let inputs =
            geojson::de::deserialize_feature_collection_str_to_vec::<GeoJsonLineString>(&req.input)
                .map_err(err_to_js)?;
        let num_inputs = inputs.len();

        let mut timer = Timer::new("snap and buffer routes", progress_cb);
        timer.step(format!("snap {num_inputs} routes"));
        let mut routes = Vec::new();
        for (idx, mut input) in inputs.into_iter().enumerate() {
            timer.log(format!("route {} / {num_inputs}", idx + 1));
            self.graph
                .mercator
                .to_mercator_in_place(&mut input.geometry);
            match self.graph.snap_route(&input.geometry, profile) {
                Ok(route) => routes.push(route),
                Err(err) => log::warn!("Couldn't snap a route: {err}"),
            }
        }

        timer.step(format!("buffer {} routes", routes.len()));
        let start_time = NaiveTime::parse_from_str(&req.start_time, "%H:%M").map_err(err_to_js)?;
        let limit = Duration::from_secs(req.max_seconds);

        let result = self
            .buffer_routes(routes, profile, start_time, limit)
            .map_err(err_to_js);
        timer.done();
        result
    }
}

// Non WASM methods, also used by the CLI
impl MapModel {
    fn parse_profile(&self, name: &str) -> Result<ProfileID, JsValue> {
        if let Some(id) = self.graph.profile_names.get(name) {
            Ok(*id)
        } else {
            Err(JsValue::from_str(&format!("unknown profile {name}")))
        }
    }

    pub async fn create(
        input_bytes: &[u8],
        gtfs_url: Option<String>,
        population_url: Option<String>,
        timer: &mut Timer,
    ) -> anyhow::Result<MapModel> {
        let mut amenities = Amenities::new();
        let mut graph = Graph::new(
            input_bytes,
            &mut amenities,
            vec![
                graph::muv_profiles::muv_car_profile(),
                graph::muv_profiles::muv_bicycle_profile(),
                graph::muv_profiles::muv_pedestrian_profile(),
            ],
            timer,
        )?;

        graph
            .setup_gtfs(
                match gtfs_url {
                    Some(url) => graph::GtfsSource::Geomedea(url),
                    None => graph::GtfsSource::None,
                },
                graph.profile_names["foot"],
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
        let profile = self.parse_profile(&req.profile)?;
        let start = self.graph.snap_to_road(
            self.graph.mercator.pt_to_mercator(Coord {
                x: req.x1,
                y: req.y1,
            }),
            profile,
        );
        let end = self.graph.snap_to_road(
            self.graph.mercator.pt_to_mercator(Coord {
                x: req.x2,
                y: req.y2,
            }),
            profile,
        );

        if req.transit {
            assert_eq!(self.graph.walking_profile_for_transit, Some(profile));
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
            let linestring = self.graph.routers[profile.0]
                .route(&self.graph, start, end)
                .map_err(err_to_js)?
                .linestring(&self.graph);
            let mut f = Feature::from(Geometry::from(&self.graph.mercator.to_wgs84(&linestring)));
            f.set_property("kind", "road");
            Ok(serde_json::to_string(&GeoJson::from(vec![f])).map_err(err_to_js)?)
        }
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }
}

#[derive(Deserialize)]
pub struct IsochroneRequest {
    // TODO Rename lon, lat to be clear?
    x: f64,
    y: f64,
    profile: String,
    transit: bool,
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
    pub profile: String,
    pub transit: bool,
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
    pub profile: String,
    pub transit: bool,
    pub use_heuristic: bool,
    pub start_time: String,
    pub max_seconds: u64,
}

#[derive(Deserialize)]
pub struct ScoreRequest {
    profile: String,
    poi_kinds: Vec<String>,
    max_seconds: u64,
}

#[derive(Deserialize)]
struct SnapRouteRequest {
    input: String,
    profile: String,
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
