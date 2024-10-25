use std::cell::RefCell;

use anyhow::{bail, Result};
use fast_paths::{deserialize_32, serialize_32, FastGraph, InputGraph, PathCalculator};
use geo::{Coord, LineString};
use itertools::Itertools;
use rstar::{primitives::GeomWithData, RTree};
use serde::{Deserialize, Serialize};
use utils::{deserialize_nodemap, LineSplit, NodeMap};

use crate::{Direction, Graph, IntersectionID, PathStep, Position, ProfileID, Road, RoadID};

/// Manages routing queries for one profile. This structure uses contraction hierarchies to calculate
/// routes very quickly. They are slower to construct, but fast to query.
#[derive(Serialize, Deserialize)]
pub struct Router {
    #[serde(deserialize_with = "deserialize_nodemap")]
    node_map: NodeMap<IntersectionID>,
    #[serde(serialize_with = "serialize_32", deserialize_with = "deserialize_32")]
    ch: FastGraph,
    #[serde(skip_serializing, skip_deserializing)]
    path_calc: RefCell<Option<PathCalculator>>,

    pub closest_road: RTree<EdgeLocation>,
}

pub type EdgeLocation = GeomWithData<LineString, RoadID>;

/// A route between two positions.
pub struct Route {
    pub start: Position,
    pub end: Position,
    pub steps: Vec<PathStep>,
}

impl Router {
    /// Creates a router for a profile. This is slow to calculate, as it builds a
    /// contraction hierarchy.
    pub fn new(roads: &Vec<Road>, profile: ProfileID) -> Self {
        let mut input_graph = InputGraph::new();
        let mut node_map = NodeMap::new();

        for road in roads {
            let cost = road.cost[profile.0].as_millis() as usize;
            let node1 = node_map.get_or_insert(road.src_i);
            let node2 = node_map.get_or_insert(road.dst_i);

            // Loops aren't ever part of a shortest path, and fast_paths warns loudly, so just skip
            if node1 == node2 {
                continue;
            }

            if road.allows_forwards(profile) {
                input_graph.add_edge(node1, node2, cost);
            }
            if road.allows_backwards(profile) {
                input_graph.add_edge(node2, node1, cost);
            }
        }
        input_graph.freeze();
        let ch = fast_paths::prepare(&input_graph);

        let path_calc = RefCell::new(Some(fast_paths::create_calculator(&ch)));

        let closest_road = RTree::bulk_load(
            roads
                .iter()
                .filter(|r| r.access[profile.0] != Direction::None)
                .map(|r| EdgeLocation::new(r.linestring.clone(), r.id))
                .collect(),
        );

        Self {
            node_map,
            ch,
            path_calc,

            closest_road,
        }
    }

    /// Calculates a route between two positions.
    pub fn route(&self, graph: &Graph, start: Position, end: Position) -> Result<Route> {
        debug!("route from {start:?} to {end:?}");
        if start == end {
            bail!("start = end");
        }

        if start.road == end.road {
            debug!("path: start = end road case");
            return Ok(Route {
                start,
                end,
                steps: vec![PathStep::Road {
                    road: start.road,
                    forwards: start.fraction_along < end.fraction_along,
                }],
            });
        }

        if start.intersection == end.intersection {
            let common_intersection = start.intersection;
            debug!("path: start = end intersection case");
            let start_road = &graph.roads[start.road.0];
            let end_road = &graph.roads[start.road.0];
            return Ok(Route {
                start,
                end,
                steps: vec![
                    PathStep::Road {
                        road: start.road,
                        forwards: start_road.dst_i == common_intersection,
                    },
                    PathStep::Road {
                        road: end.road,
                        forwards: end_road.src_i == common_intersection,
                    },
                ],
            });
        }

        let start_node = self.node_map.get(start.intersection).unwrap();
        let end_node = self.node_map.get(end.intersection).unwrap();

        let Some(path) = self
            .path_calc
            .borrow_mut()
            // This'll be empty right after loading a serialized Graph
            .get_or_insert_with(|| fast_paths::create_calculator(&self.ch))
            .calc_path(&self.ch, start_node, end_node)
        else {
            bail!("No path");
        };

        let mut steps = Vec::new();
        for (pos, pair) in path.get_nodes().windows(2).with_position() {
            let i1 = self.node_map.translate_id(pair[0]);
            let i2 = self.node_map.translate_id(pair[1]);
            let Some(road) = graph.find_edge(i1, i2) else {
                bail!("No road between {i1:?} and {i2:?}");
            };

            if (pos == itertools::Position::First || pos == itertools::Position::Only)
                && road.id != start.road
            {
                steps.push(PathStep::Road {
                    road: start.road,
                    // TODO Test carefully.
                    forwards: start.fraction_along > 0.5,
                });
            }
            steps.push(PathStep::Road {
                road: road.id,
                forwards: road.src_i == i1,
            });
            if (pos == itertools::Position::Last || pos == itertools::Position::Only)
                && road.id != end.road
            {
                steps.push(PathStep::Road {
                    road: end.road,
                    // TODO Test carefully.
                    forwards: end.fraction_along <= 0.5,
                });
            }
        }

        Ok(Route { start, end, steps })
    }
}

impl Route {
    /// Renders a route as a linestring (in Mercator), with precise positions at the start and end.
    pub fn linestring(&self, graph: &Graph) -> LineString {
        let mut pts = Vec::new();
        debug!("turning {} steps into linestring", self.steps.len());
        debug!("route start is {:?}, end is {:?}", self.start, self.end);
        for (pos, step) in self.steps.iter().with_position() {
            match step {
                PathStep::Road { road, forwards } => {
                    debug!("step on {road:?}, forwards = {forwards}");
                    pts.extend(slice_road_step(
                        &graph.roads[road.0].linestring,
                        *forwards,
                        &self.start,
                        &self.end,
                        pos,
                    ));
                }
                PathStep::Transit { .. } => unreachable!(),
            }
        }
        pts.dedup();
        LineString::new(pts)
    }
}

fn slice_road_step(
    linestring: &LineString,
    forwards: bool,
    start: &Position,
    end: &Position,
    pos: itertools::Position,
) -> Vec<Coord> {
    let mut pts = match pos {
        itertools::Position::First => {
            let (a, b) = if forwards {
                (start.fraction_along, 1.0)
            } else {
                (0.0, start.fraction_along)
            };
            linestring
                .line_split_twice(a, b)
                .unwrap()
                .into_second()
                // TODO Workaround some crashes in Severance Snape, where the first PathStep seems
                // to start on a road in reverse and immediately go somewhere else
                .map(|ls| ls.0)
                .unwrap_or_else(Vec::new)
        }
        itertools::Position::Last => {
            let (a, b) = if forwards {
                (0.0, end.fraction_along)
            } else {
                (end.fraction_along, 1.0)
            };
            linestring
                .line_split_twice(a, b)
                .unwrap()
                .into_second()
                .map(|ls| ls.0)
                .unwrap_or_else(Vec::new)
        }
        itertools::Position::Middle => linestring.0.clone(),
        itertools::Position::Only => linestring
            .line_split_twice(start.fraction_along, end.fraction_along)
            .unwrap()
            .into_second()
            .map(|ls| ls.0)
            .unwrap_or_else(Vec::new),
    };
    if !forwards {
        pts.reverse();
    }
    pts
}
