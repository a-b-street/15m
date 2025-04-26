use std::collections::HashMap;

use anyhow::Result;
use geo::{Coord, Point};
use geojson::{Feature, GeoJson, Geometry};
use graph::{Graph, Timer};
use osm_reader::OsmID;
use serde::{Deserialize, Serialize};
use utils::{Mercator, Tags};

#[derive(Serialize, Deserialize)]
pub struct Amenities {
    pub amenities: Vec<Amenity>,
    // Indexed by RoadID, then by ProfileID. Each amenity could snap to different roads depending
    // on the profile.
    pub per_road: Vec<Vec<Vec<AmenityID>>>,
}

#[derive(Serialize, Deserialize)]
pub struct Amenity {
    pub id: AmenityID,
    pub osm_id: OsmID,
    pub point: Point,
    pub kind: String,
    pub name: Option<String>,

    // Supporting details for some cases only
    pub brand: Option<String>,
    pub cuisine: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AmenityID(pub usize);

impl Amenities {
    pub fn new() -> Self {
        Self {
            amenities: Vec::new(),
            per_road: Vec::new(),
        }
    }

    pub fn finalize(&mut self, graph: &Graph, timer: &mut Timer) {
        timer.step("snap amenities");
        self.per_road = std::iter::repeat_with(|| {
            std::iter::repeat_with(Vec::new)
                .take(graph.profile_names.len())
                .collect()
        })
        .take(graph.roads.len())
        .collect();

        for amenity in &mut self.amenities {
            amenity.point = graph.mercator.pt_to_mercator(amenity.point.into()).into();

            for (idx, router) in graph.routers.iter().enumerate() {
                if let Some(r) = router.closest_road.nearest_neighbor(&amenity.point) {
                    self.per_road[r.data.0][idx].push(amenity.id);
                }
            }
        }
    }

    pub fn render_amenities(&self, mercator: &Mercator) -> Result<String> {
        let mut features = Vec::new();
        for a in &self.amenities {
            features.push(a.to_gj(mercator));
        }
        let gj = GeoJson::from(features);
        let out = serde_json::to_string(&gj)?;
        Ok(out)
    }
}

impl Amenity {
    pub fn maybe_new(tags: &Tags, osm_id: OsmID, point: Point, id: AmenityID) -> Option<Self> {
        let kind = Self::is_amenity(tags)?;
        Some(Self {
            id,
            osm_id,
            point,
            name: tags.get("name").cloned(),
            kind,
            brand: tags.get("brand").cloned(),
            cuisine: tags.get("cuisine").cloned(),
        })
    }

    pub fn to_gj(&self, mercator: &Mercator) -> Feature {
        let mut f = Feature::from(Geometry::from(&mercator.to_wgs84(&self.point)));
        f.set_property("amenity_kind", self.kind.clone());
        f.set_property("osm_id", self.osm_id.to_string());
        if let Some(ref name) = self.name {
            f.set_property("name", name.clone());
        }
        if let Some(ref brand) = self.brand {
            f.set_property("brand", brand.clone());
        }
        if let Some(ref cuisine) = self.cuisine {
            f.set_property("cuisine", cuisine.clone());
        }
        f
    }

    /// Determines if this OSM object should count as some kind of useful commercial amenity. Many
    /// categories are excluded. Returns the category.
    fn is_amenity(tags: &Tags) -> Option<String> {
        // TODO Allowlist might be easier
        if tags.is_any(
            "amenity",
            vec![
                "atm",
                "bench",
                "bicycle_rental",
                "bicycle_repair_station",
                "car_rental",
                "car_sharing",
                "car_wash",
                "charging_station",
                "dog_litter_box",
                "drinking_water",
                "fuel",
                "grit_bin",
                "housing_office",
                "motorcycle_parking",
                "parcel_locker",
                "parking",
                "parking_entrance",
                "parking_meter",
                "parking_space",
                "post_box",
                "public_bookcase",
                "recycling",
                "taxi",
                "telephone",
                "toilets",
                "vending_machine",
                "waste_basket",
                "waste_disposal",
            ],
        ) {
            return None;
        }

        tags.get("amenity").or_else(|| tags.get("shop")).cloned()
    }
}

impl utils::osm2graph::OsmReader for Amenities {
    fn node(&mut self, id: osm_reader::NodeID, pt: Coord, tags: Tags) {
        self.amenities.extend(Amenity::maybe_new(
            &tags,
            OsmID::Node(id),
            pt.into(),
            AmenityID(self.amenities.len()),
        ));
    }

    fn way(
        &mut self,
        id: osm_reader::WayID,
        node_ids: &Vec<osm_reader::NodeID>,
        node_mapping: &HashMap<osm_reader::NodeID, Coord>,
        tags: &Tags,
    ) {
        self.amenities.extend(Amenity::maybe_new(
            tags,
            OsmID::Way(id),
            // TODO Centroid
            node_mapping[&node_ids[0]].into(),
            AmenityID(self.amenities.len()),
        ));
    }

    // TODO Are there amenities as relations?
    fn relation(
        &mut self,
        _id: osm_reader::RelationID,
        _members: &Vec<(String, OsmID)>,
        _tags: &Tags,
    ) {
    }
}
