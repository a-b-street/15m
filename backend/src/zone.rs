use anyhow::Result;
use flatgeobuf::{FeatureProperties, FgbFeature, GeozeroGeometry, HttpFgbReader};
use geo::{Area, MultiPolygon, Polygon};
use geojson::{Feature, FeatureCollection, Geometry};
use rstar::{primitives::GeomWithData, RTree};
use serde::{Deserialize, Serialize};
use utils::Mercator;

use crate::Timer;

#[derive(Serialize, Deserialize)]
pub struct Zones {
    pub zones: Vec<Zone>,
    pub rtree: RTree<GeomWithData<Polygon, ZoneID>>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ZoneID(pub usize);

#[derive(Serialize, Deserialize)]
pub struct Zone {
    // TODO Maybe split these upfront, including area and population, and just store in the RTree?
    // TODO Do these need to be Mercator?
    pub geom: MultiPolygon,
    // TODO Later on, this could be generic or user-supplied
    pub population: u32,
    pub area_km2: f64,
}

impl Zones {
    pub async fn load(
        population_url: Option<String>,
        mercator: &Mercator,
        timer: &mut Timer,
    ) -> Result<Self> {
        let zones = if let Some(url) = population_url {
            timer.step("load population zones");
            load_zones(url, mercator).await?
        } else {
            Vec::new()
        };
        let mut zone_objects = Vec::new();
        for (idx, zone) in zones.iter().enumerate() {
            let id = ZoneID(idx);
            // MultiPolygon isn't supported, so just insert multiple
            for polygon in &zone.geom {
                zone_objects.push(GeomWithData::new(polygon.clone(), id));
            }
        }
        Ok(Self {
            zones,
            rtree: RTree::bulk_load(zone_objects),
        })
    }

    /// Returns a GeoJSON string
    pub fn render_zones(&self, mercator: &Mercator) -> Result<String> {
        let mut features = Vec::new();
        let mut max_density: f64 = 0.0;
        for zone in &self.zones {
            let mut f = Feature::from(Geometry::from(&mercator.to_wgs84(&zone.geom)));
            f.set_property("population", zone.population);
            let density = (zone.population as f64) / zone.area_km2;
            f.set_property("density", density);
            features.push(f);

            max_density = max_density.max(density);
        }
        Ok(serde_json::to_string(&FeatureCollection {
            features,
            bbox: None,
            foreign_members: Some(
                serde_json::json!({
                    "max_density": max_density,
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        })?)
    }
}

async fn load_zones(url: String, mercator: &Mercator) -> Result<Vec<Zone>> {
    let bbox = mercator.wgs84_bounds;
    let mut fgb = HttpFgbReader::open(&url)
        .await?
        .select_bbox(bbox.min().x, bbox.min().y, bbox.max().x, bbox.max().y)
        .await?;

    let mut zones = Vec::new();
    while let Some(feature) = fgb.next().await? {
        // TODO Could intersect with boundary_polygon, but some extras nearby won't hurt anything
        let mut geom = get_multipolygon(feature)?;
        mercator.to_mercator_in_place(&mut geom);
        let area_km2 = 1e-6 * geom.unsigned_area();
        // TODO Re-encode as UInt
        let population = feature.property::<i64>("population")?.try_into()?;

        zones.push(Zone {
            geom,
            population,
            area_km2,
        });
    }
    Ok(zones)
}

fn get_multipolygon(f: &FgbFeature) -> Result<MultiPolygon> {
    let mut p = geozero::geo_types::GeoWriter::new();
    f.process_geom(&mut p)?;
    match p.take_geometry().unwrap() {
        geo::Geometry::Polygon(p) => Ok(MultiPolygon(vec![p])),
        geo::Geometry::MultiPolygon(mp) => Ok(mp),
        _ => bail!("Wrong type in fgb"),
    }
}
