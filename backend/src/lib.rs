//#[macro_use]
//extern crate log;

use std::fmt;
use std::sync::Once;

use geo::{Coord, LineString, Point, Polygon};
use geojson::{Feature, GeoJson, Geometry};
use rstar::{primitives::GeomWithData, RTree};
use serde::{Deserialize, Serialize};
use utils::{Mercator, Tags};
use wasm_bindgen::prelude::*;

mod isochrone;
mod scrape;

static START: Once = Once::new();

#[wasm_bindgen]
pub struct MapModel {
    roads: Vec<Road>,
    intersections: Vec<Intersection>,
    // All geometry stored in worldspace, including rtrees
    mercator: Mercator,
    closest_intersection: RTree<IntersectionLocation>,
    boundary_polygon: Polygon,
}

type IntersectionLocation = GeomWithData<[f64; 2], IntersectionID>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct RoadID(pub usize);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct IntersectionID(pub usize);

impl fmt::Display for RoadID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Road #{}", self.0)
    }
}

impl fmt::Display for IntersectionID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Intersection #{}", self.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Forwards,
    Backwards,
    Both,
    None,
}

#[derive(Clone, Copy)]
pub enum Mode {
    Car,
    Bicycle,
    Foot,
}

pub struct Road {
    id: RoadID,
    src_i: IntersectionID,
    dst_i: IntersectionID,
    way: osm_reader::WayID,
    node1: osm_reader::NodeID,
    node2: osm_reader::NodeID,
    linestring: LineString,
    tags: Tags,

    // A simplified view of who can access a road. All might be None (buses, trains ignored)
    // TODO enum map?
    access_car: Direction,
    access_bicycle: Direction,
    access_foot: Direction,
}

pub struct Intersection {
    id: IntersectionID,
    #[allow(dead_code)]
    node: osm_reader::NodeID,
    point: Point,
    roads: Vec<RoadID>,
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

        scrape::scrape_osm(input_bytes).map_err(err_to_js)
    }

    /// Returns a GeoJSON string. Just shows the full network
    #[wasm_bindgen()]
    pub fn render(&self) -> Result<String, JsValue> {
        let mut features = Vec::new();

        for r in &self.roads {
            features.push(r.to_gj(&self.mercator));
        }

        let gj = GeoJson::from(features);
        let out = serde_json::to_string(&gj).map_err(err_to_js)?;
        Ok(out)
    }

    /// Return a polygon covering the world, minus a hole for the boundary, in WGS84
    #[wasm_bindgen(js_name = getInvertedBoundary)]
    pub fn get_inverted_boundary(&self) -> Result<String, JsValue> {
        let (boundary, _) = self.mercator.to_wgs84(&self.boundary_polygon).into_inner();
        let polygon = Polygon::new(
            LineString::from(vec![
                (180.0, 90.0),
                (-180.0, 90.0),
                (-180.0, -90.0),
                (180.0, -90.0),
                (180.0, 90.0),
            ]),
            vec![boundary],
        );
        let f = Feature::from(Geometry::from(&polygon));
        let out = serde_json::to_string(&f).map_err(err_to_js)?;
        Ok(out)
    }

    #[wasm_bindgen(js_name = getBounds)]
    pub fn get_bounds(&self) -> Vec<f64> {
        let b = &self.mercator.wgs84_bounds;
        vec![b.min().x, b.min().y, b.max().x, b.max().y]
    }

    #[wasm_bindgen(js_name = isochrone)]
    pub fn isochrone(&self, input: JsValue) -> Result<String, JsValue> {
        let req: IsochroneRequest = serde_wasm_bindgen::from_value(input)?;
        let start = self.mercator.pt_to_mercator(Coord { x: req.x, y: req.y });
        let mode = match req.mode.as_str() {
            "car" => Mode::Car,
            "bicycle" => Mode::Bicycle,
            "foot" => Mode::Foot,
            // TODO error plumbing
            x => panic!("bad input {x}"),
        };
        isochrone::calculate(&self, start, mode).map_err(err_to_js)
    }
}

impl MapModel {
    fn roads_per_intersection(&self, i: IntersectionID, mode: Mode) -> impl Iterator<Item = &Road> {
        self.intersections[i.0]
            .roads
            .iter()
            .map(|r| &self.roads[r.0])
            .filter(move |r| r.allows_forwards(mode) || r.allows_backwards(mode))
    }
}

impl Road {
    fn allows_forwards(&self, mode: Mode) -> bool {
        let dir = match mode {
            Mode::Car => self.access_car,
            Mode::Bicycle => self.access_bicycle,
            Mode::Foot => self.access_foot,
        };
        matches!(dir, Direction::Forwards | Direction::Both)
    }

    fn allows_backwards(&self, mode: Mode) -> bool {
        let dir = match mode {
            Mode::Car => self.access_car,
            Mode::Bicycle => self.access_bicycle,
            Mode::Foot => self.access_foot,
        };
        matches!(dir, Direction::Backwards | Direction::Both)
    }

    fn to_gj(&self, mercator: &Mercator) -> Feature {
        let mut f = Feature::from(Geometry::from(&mercator.to_wgs84(&self.linestring)));
        // TODO Rethink most of this -- it's debug info
        f.set_property("id", self.id.0);
        f.set_property("way", self.way.to_string());
        f.set_property("node1", self.node1.to_string());
        f.set_property("node2", self.node2.to_string());
        for (k, v) in &self.tags.0 {
            f.set_property(k, v.to_string());
        }
        f.set_property("access_car", format!("{:?}", self.access_car));
        f.set_property("access_bicycle", format!("{:?}", self.access_bicycle));
        f.set_property("access_foot", format!("{:?}", self.access_foot));
        f
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
