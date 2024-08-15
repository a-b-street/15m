use std::cell::RefCell;

use anyhow::{bail, Result};
use fast_paths::{deserialize_32, serialize_32, FastGraph, InputGraph, PathCalculator};
use geo::{Coord, LineString};
use geojson::{Feature, GeoJson, Geometry};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use utils::{deserialize_nodemap, LineSplit, NodeMap};

use crate::costs::cost;
use crate::graph::{Graph, IntersectionID, Mode, Position, Road, RoadID};
use crate::gtfs::{StopID, TripID};

#[derive(Serialize, Deserialize)]
pub struct Router {
    #[serde(deserialize_with = "deserialize_nodemap")]
    node_map: NodeMap<IntersectionID>,
    #[serde(serialize_with = "serialize_32", deserialize_with = "deserialize_32")]
    ch: FastGraph,
    #[serde(skip_serializing, skip_deserializing)]
    path_calc: RefCell<Option<PathCalculator>>,
}

impl Router {
    pub fn new(roads: &Vec<Road>, mode: Mode) -> Self {
        let mut input_graph = InputGraph::new();
        let mut node_map = NodeMap::new();

        for road in roads {
            let cost = cost(road, mode).as_millis() as usize;
            let node1 = node_map.get_or_insert(road.src_i);
            let node2 = node_map.get_or_insert(road.dst_i);

            if road.allows_forwards(mode) {
                input_graph.add_edge(node1, node2, cost);
            }
            if road.allows_backwards(mode) {
                input_graph.add_edge(node2, node1, cost);
            }
        }
        input_graph.freeze();
        let ch = fast_paths::prepare(&input_graph);

        let path_calc = RefCell::new(Some(fast_paths::create_calculator(&ch)));

        Self {
            node_map,
            ch,
            path_calc,
        }
    }

    // TODO This doesn't handle start=end cases
    pub fn route_steps(
        &self,
        graph: &Graph,
        start: Position,
        end: Position,
    ) -> Result<Vec<PathStep>> {
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
            let road = graph.find_edge(i1, i2);

            if pos == itertools::Position::First && road.id != start.road {
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
            if pos == itertools::Position::Last && road.id != end.road {
                steps.push(PathStep::Road {
                    road: end.road,
                    // TODO Test carefully.
                    forwards: end.fraction_along <= 0.5,
                });
            }
        }

        Ok(steps)
    }

    // TODO Rename -- renders to GJ
    pub fn route(&self, graph: &Graph, start: Position, end: Position) -> Result<String> {
        if start == end {
            bail!("start = end");
        }
        if start.road == end.road {
            // Just slice the one road
            let mut slice = graph.roads[start.road.0]
                .linestring
                .line_split_twice(start.fraction_along, end.fraction_along)
                .unwrap()
                .into_second()
                .unwrap();
            if start.fraction_along > end.fraction_along {
                slice.0.reverse();
            }
            let mut f = Feature::from(Geometry::from(&graph.mercator.to_wgs84(&slice)));
            f.set_property("kind", "road");
            return Ok(serde_json::to_string(&GeoJson::from(vec![f]))?);
        }

        let steps = self.route_steps(graph, start, end)?;

        // TODO Share code with PT?
        let mut pts = Vec::new();
        for (pos, step) in steps.into_iter().with_position() {
            match step {
                PathStep::Road { road, forwards } => {
                    pts.extend(slice_road_step(
                        &graph.roads[road.0].linestring,
                        forwards,
                        &start,
                        &end,
                        pos,
                    ));
                }
                PathStep::Transit { .. } => unreachable!(),
            }
        }
        pts.dedup();

        let mut f = Feature::from(Geometry::from(
            &graph.mercator.to_wgs84(&LineString::new(pts)),
        ));
        f.set_property("kind", "road");
        Ok(serde_json::to_string(&GeoJson::from(vec![f]))?)
    }
}

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
                .unwrap()
                .0
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
                .unwrap()
                .0
        }
        _ => linestring.0.clone(),
    };
    if !forwards {
        pts.reverse();
    }
    pts
}
