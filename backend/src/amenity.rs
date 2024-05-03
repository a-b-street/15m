use geo::Point;
use geojson::{Feature, Geometry};
use osm_reader::OsmID;
use utils::{Mercator, Tags};

pub struct Amenity {
    pub osm_id: OsmID,
    pub point: Point,
    pub kind: String,
    pub name: Option<String>,

    // Supporting details for some cases only
    pub brand: Option<String>,
    pub cuisine: Option<String>,
}

impl Amenity {
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
    pub fn is_amenity(tags: &Tags) -> Option<String> {
        // TODO Allowlist might be easier
        if tags.is_any(
            "amenity",
            vec![
                "atm",
                "bench",
                "bicycle_parking",
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
