use anyhow::Result;
use enum_map::{Enum, EnumMap};
use geo::{LineString, Point, Polygon};
use geojson::{Feature, GeoJson, Geometry};
use rstar::{primitives::GeomWithData, RTree};
use utils::{Mercator, Tags};

use crate::amenity::Amenity;

pub struct Graph {
    pub roads: Vec<Road>,
    pub intersections: Vec<Intersection>,
    // All geometry stored in worldspace, including rtrees
    pub mercator: Mercator,
    pub closest_intersection: RTree<IntersectionLocation>,
    pub boundary_polygon: Polygon,

    // Unrelated to the transportation graph above. Maybe should be more separate.
    pub amenities: Vec<Amenity>,
}

pub type IntersectionLocation = GeomWithData<[f64; 2], IntersectionID>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct RoadID(pub usize);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct IntersectionID(pub usize);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct AmenityID(pub usize);

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Forwards,
    Backwards,
    Both,
    None,
}

#[derive(Clone, Copy, Enum)]
pub enum Mode {
    Car,
    Bicycle,
    Foot,
}

pub struct Road {
    pub id: RoadID,
    pub src_i: IntersectionID,
    pub dst_i: IntersectionID,
    pub way: osm_reader::WayID,
    pub node1: osm_reader::NodeID,
    pub node2: osm_reader::NodeID,
    pub linestring: LineString,
    pub tags: Tags,

    // A simplified view of who can access a road. All might be None (buses, trains ignored)
    pub access: EnumMap<Mode, Direction>,

    // These're broken down this way because the 3 graphs look different and could snap to
    // different roads in each case
    pub amenities: EnumMap<Mode, Vec<AmenityID>>,
}

pub struct Intersection {
    pub id: IntersectionID,
    #[allow(dead_code)]
    pub node: osm_reader::NodeID,
    pub point: Point,
    pub roads: Vec<RoadID>,
}

impl Graph {
    /// Call with bytes of an osm.pbf or osm.xml string
    pub fn new(input_bytes: &[u8]) -> Result<Graph> {
        // TODO make a method there
        crate::scrape::scrape_osm(input_bytes)
    }

    /// Returns a GeoJSON string. Just shows the full network and amenities
    pub fn render(&self) -> Result<String> {
        let mut features = Vec::new();

        for r in &self.roads {
            features.push(r.to_gj(&self.mercator));
        }
        for a in &self.amenities {
            features.push(a.to_gj(&self.mercator));
        }

        let gj = GeoJson::from(features);
        let out = serde_json::to_string(&gj)?;
        Ok(out)
    }

    /// Return a polygon covering the world, minus a hole for the boundary, in WGS84
    pub fn get_inverted_boundary(&self) -> Result<String> {
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
        let out = serde_json::to_string(&f)?;
        Ok(out)
    }

    pub fn roads_per_intersection(
        &self,
        i: IntersectionID,
        mode: Mode,
    ) -> impl Iterator<Item = &Road> {
        self.intersections[i.0]
            .roads
            .iter()
            .map(|r| &self.roads[r.0])
            .filter(move |r| r.allows_forwards(mode) || r.allows_backwards(mode))
    }
}

impl Road {
    pub fn allows_forwards(&self, mode: Mode) -> bool {
        matches!(self.access[mode], Direction::Forwards | Direction::Both)
    }

    pub fn allows_backwards(&self, mode: Mode) -> bool {
        matches!(self.access[mode], Direction::Backwards | Direction::Both)
    }

    pub fn to_gj(&self, mercator: &Mercator) -> Feature {
        let mut f = Feature::from(Geometry::from(&mercator.to_wgs84(&self.linestring)));
        // TODO Rethink most of this -- it's debug info
        f.set_property("id", self.id.0);
        f.set_property("way", self.way.to_string());
        f.set_property("node1", self.node1.to_string());
        f.set_property("node2", self.node2.to_string());
        for (k, v) in &self.tags.0 {
            f.set_property(k, v.to_string());
        }
        f.set_property("access_car", format!("{:?}", self.access[Mode::Car]));
        f.set_property(
            "access_bicycle",
            format!("{:?}", self.access[Mode::Bicycle]),
        );
        f.set_property("access_foot", format!("{:?}", self.access[Mode::Foot]));
        f
    }
}
