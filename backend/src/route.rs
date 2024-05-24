use std::cell::RefCell;

use anyhow::{bail, Result};
use fast_paths::{deserialize_32, serialize_32, FastGraph, InputGraph, PathCalculator};
use geojson::{Feature, GeoJson, Geometry};
use serde::{Deserialize, Serialize};
use utils::{deserialize_nodemap, NodeMap};

use crate::costs::cost;
use crate::graph::{Graph, IntersectionID, Mode, Road};

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

    pub fn route(
        &self,
        graph: &Graph,
        start: IntersectionID,
        end: IntersectionID,
    ) -> Result<String> {
        if start == end {
            bail!("start = end");
        }
        let start = self.node_map.get(start).unwrap();
        let end = self.node_map.get(end).unwrap();

        let Some(path) = self
            .path_calc
            .borrow_mut()
            // This'll be empty right after loading a serialized Graph
            .get_or_insert_with(|| fast_paths::create_calculator(&self.ch))
            .calc_path(&self.ch, start, end)
        else {
            bail!("No path");
        };

        // TODO Ideally glue together one LineString
        let mut features = Vec::new();
        for pair in path.get_nodes().windows(2) {
            let i1 = self.node_map.translate_id(pair[0]);
            let i2 = self.node_map.translate_id(pair[1]);
            let road = graph.find_edge(i1, i2);
            let mut f = Feature::from(Geometry::from(&graph.mercator.to_wgs84(&road.linestring)));
            f.set_property("kind", "road");
            features.push(f);
        }
        Ok(serde_json::to_string(&GeoJson::from(features))?)
    }
}
