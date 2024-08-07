use anyhow::Result;
use enum_map::{Enum, EnumMap};
use geo::{Coord, LineLocatePoint, LineString, Point, Polygon};
use geojson::{Feature, GeoJson, Geometry};
use rstar::{primitives::GeomWithData, RTree};
use serde::{Deserialize, Serialize};
use utils::Mercator;

use crate::amenity::Amenity;
use crate::gtfs::{GtfsModel, StopID};
use crate::route::Router;

#[derive(Serialize, Deserialize)]
pub struct Graph {
    pub roads: Vec<Road>,
    pub intersections: Vec<Intersection>,
    // All geometry stored in worldspace, including rtrees
    pub mercator: Mercator,
    pub closest_road: EnumMap<Mode, RTree<EdgeLocation>>,
    pub router: EnumMap<Mode, Router>,
    pub boundary_polygon: Polygon,

    // Unrelated to the transportation graph above. Maybe should be more separate.
    pub amenities: Vec<Amenity>,

    pub gtfs: GtfsModel,
}

pub type EdgeLocation = GeomWithData<LineString, RoadID>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RoadID(pub usize);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IntersectionID(pub usize);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AmenityID(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    Forwards,
    Backwards,
    Both,
    None,
}

// TODO Justify why PublicTransit isn't captured here
#[derive(Clone, Copy, Enum, Debug, Serialize, Deserialize)]
pub enum Mode {
    Car,
    Bicycle,
    Foot,
}

#[derive(Serialize, Deserialize)]
pub struct Road {
    pub id: RoadID,
    pub src_i: IntersectionID,
    pub dst_i: IntersectionID,
    pub way: osm_reader::WayID,
    pub node1: osm_reader::NodeID,
    pub node2: osm_reader::NodeID,
    // For performance
    pub length_meters: f64,
    pub linestring: LineString,

    // A simplified view of who can access a road. All might be None (buses, trains ignored)
    pub access: EnumMap<Mode, Direction>,

    // These're broken down this way because the 3 graphs look different and could snap to
    // different roads in each case
    pub amenities: EnumMap<Mode, Vec<AmenityID>>,

    // Meters/second, for cars
    pub max_speed: f64,

    pub stops: Vec<StopID>,
}

#[derive(Serialize, Deserialize)]
pub struct Intersection {
    pub id: IntersectionID,
    #[allow(dead_code)]
    pub node: osm_reader::NodeID,
    pub point: Point,
    pub roads: Vec<RoadID>,
}

impl Graph {
    /// Returns a GeoJSON string. Just shows the full network and amenities
    pub fn render_debug(&self) -> Result<String> {
        let mut features = Vec::new();

        for r in &self.roads {
            features.push(r.to_gj(&self.mercator));
        }
        for a in &self.amenities {
            features.push(a.to_gj(&self.mercator));
        }
        for s in &self.gtfs.stops {
            features.push(s.to_gj(&self.mercator));
        }

        let gj = GeoJson::from(features);
        let out = serde_json::to_string(&gj)?;
        Ok(out)
    }

    pub fn render_amenities(&self) -> Result<String> {
        let mut features = Vec::new();
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

    pub fn find_edge(&self, i1: IntersectionID, i2: IntersectionID) -> &Road {
        // TODO Store lookup table
        for r in &self.intersections[i1.0].roads {
            let road = &self.roads[r.0];
            if road.src_i == i2 || road.dst_i == i2 {
                return road;
            }
        }
        panic!("no road from {i1:?} to {i2:?} or vice versa");
    }

    pub fn snap_to_road(&self, pt: Coord, mode: Mode) -> Position {
        let r = self.closest_road[mode]
            .nearest_neighbor(&pt.into())
            .unwrap()
            .data;
        let road = &self.roads[r.0];
        let fraction_along = road.linestring.line_locate_point(&pt.into()).unwrap();
        let intersection = if fraction_along <= 0.5 {
            road.src_i
        } else {
            road.dst_i
        };
        Position {
            road: road.id,
            fraction_along,
            intersection,
        }
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
        f.set_property("access_car", format!("{:?}", self.access[Mode::Car]));
        f.set_property(
            "access_bicycle",
            format!("{:?}", self.access[Mode::Bicycle]),
        );
        f.set_property("access_foot", format!("{:?}", self.access[Mode::Foot]));
        f.set_property("max_speed_mph", self.max_speed * 2.23694);
        f
    }
}

/// A position along a road, along with the closer intersection
#[derive(PartialEq)]
pub struct Position {
    pub road: RoadID,
    pub fraction_along: f64,
    pub intersection: IntersectionID,
}
