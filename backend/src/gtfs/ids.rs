use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// The full string IDs used in GTFS
pub mod orig_ids {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct StopID(String);

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct TripID(String);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StopID(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TripID(pub usize);

impl CheapID for StopID {
    fn new(x: usize) -> Self {
        Self(x)
    }
}
impl CheapID for TripID {
    fn new(x: usize) -> Self {
        Self(x)
    }
}

pub trait CheapID: Copy {
    fn new(x: usize) -> Self;
}

#[derive(Serialize, Deserialize)]
pub struct IDMapping<K: Ord, V> {
    orig_to_cheap: BTreeMap<K, V>,
    // We don't need to store the inverse. It's more convenient for each object to own that.
}

impl<K: Clone + std::fmt::Debug + Ord, V: CheapID> IDMapping<K, V> {
    pub fn new() -> Self {
        Self {
            orig_to_cheap: BTreeMap::new(),
        }
    }

    pub fn insert_new(&mut self, orig: K) -> Result<V> {
        let cheap = V::new(self.orig_to_cheap.len());
        if self.orig_to_cheap.insert(orig.clone(), cheap).is_some() {
            bail!("IDMapping::insert_new has duplicate input for {:?}", orig);
        }
        Ok(cheap)
    }

    pub fn get(&self, orig: &K) -> Option<V> {
        self.orig_to_cheap.get(orig).cloned()
    }
}
