#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

mod costs;
mod gtfs;
mod isochrone;
mod route;
mod scrape;
pub mod snap;
mod timer;
mod transit_route;

use anyhow::Result;
use enum_map::{Enum, EnumMap};
use geo::{Coord, LineLocatePoint, LineString, Point, Polygon};
use geojson::{Feature, FeatureCollection, Geometry};
use serde::{Deserialize, Serialize};
use utils::{Mercator, Tags};

pub use self::route::{Route, Router};
pub use self::timer::Timer;
pub use crate::gtfs::GtfsModel;
use crate::gtfs::{StopID, TripID};

/// A study area imported from OpenStreetMap.
#[derive(Serialize, Deserialize)]
pub struct Graph {
    pub roads: Vec<Road>,
    pub intersections: Vec<Intersection>,
    // All geometry stored in worldspace, including rtrees
    /// `Graph` stores all geometry in a Mercator projection for the study area. This field helps
    /// translation to/from WGS84.
    pub mercator: Mercator,
    pub router: EnumMap<Mode, Router>,
    /// A polygon covering the study area.
    pub boundary_polygon: Polygon,

    pub gtfs: GtfsModel,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RoadID(pub usize);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IntersectionID(pub usize);

/// How can a `Road` be crossed by a particular `Mode`?
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    Forwards,
    Backwards,
    Both,
    None,
}

/// The graph structure is expressed for each of these different modes of travel.
// TODO Justify why PublicTransit isn't captured here
#[derive(Clone, Copy, Enum, Debug, Serialize, Deserialize)]
pub enum Mode {
    Car,
    Bicycle,
    Foot,
}

impl Mode {
    /// Parses a string. Treats "transit" as Mode::Foot
    pub fn parse(x: &str) -> Result<Mode> {
        match x {
            "car" => Ok(Mode::Car),
            "bicycle" => Ok(Mode::Bicycle),
            "foot" => Ok(Mode::Foot),
            // Caller special-cases this
            "transit" => Ok(Mode::Foot),
            x => bail!("unknown Mode input {x}"),
        }
    }
}

/// Represents an edge going between exactly two `Intersection`s.
#[derive(Serialize, Deserialize)]
pub struct Road {
    pub id: RoadID,
    pub src_i: IntersectionID,
    pub dst_i: IntersectionID,
    pub way: osm_reader::WayID,
    pub node1: osm_reader::NodeID,
    pub node2: osm_reader::NodeID,
    pub osm_tags: Tags,
    // For performance
    pub length_meters: f64,
    pub linestring: LineString,

    // A simplified view of who can access a road. All might be None (buses, trains ignored)
    /// Per mode, what direction is this road traversable?
    pub access: EnumMap<Mode, Direction>,

    /// For cars, the speed limit in meters/second
    pub max_speed: f64,

    /// The bus stops associated with this road
    pub stops: Vec<StopID>,
}

/// An intersection between one or more roads. This might represent a dead-end.
#[derive(Serialize, Deserialize)]
pub struct Intersection {
    pub id: IntersectionID,
    #[allow(dead_code)]
    pub node: osm_reader::NodeID,
    pub point: Point,
    pub roads: Vec<RoadID>,
}

impl Graph {
    /// Returns GeoJSON with roads and stops
    pub fn render_debug(&self) -> FeatureCollection {
        let mut features = Vec::new();

        for r in &self.roads {
            features.push(r.to_gj(&self.mercator));
        }
        for s in &self.gtfs.stops {
            features.push(s.to_gj(&self.mercator));
        }

        FeatureCollection {
            features,
            bbox: None,
            foreign_members: None,
        }
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

    /// Find the Road going from `i1` to `i2` or vice versa
    pub fn find_edge(&self, i1: IntersectionID, i2: IntersectionID) -> Option<&Road> {
        // TODO Store lookup table
        for r in &self.intersections[i1.0].roads {
            let road = &self.roads[r.0];
            if road.src_i == i2 || road.dst_i == i2 {
                return Some(road);
            }
        }
        None
    }

    /// Given a point (in Mercator) and mode, snap to a position along some road that mode can
    /// cross.
    pub fn snap_to_road(&self, pt: Coord, mode: Mode) -> Position {
        let r = self.router[mode]
            .closest_road
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
    /// Can this mode cross this road in the forwards direction?
    pub fn allows_forwards(&self, mode: Mode) -> bool {
        matches!(self.access[mode], Direction::Forwards | Direction::Both)
    }

    /// Can this mode cross this road in the backwards direction?
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

/// A position along a road, along with the closest intersection
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Position {
    pub road: RoadID,
    pub fraction_along: f64,
    pub intersection: IntersectionID,
}

pub enum GtfsSource {
    Dir(String),
    Geomedea(String),
    None,
}

/// A single step along a route
#[derive(Debug)]
pub enum PathStep {
    Road {
        road: RoadID,
        forwards: bool,
    },
    Transit {
        stop1: StopID,
        trip: TripID,
        stop2: StopID,
    },
}
