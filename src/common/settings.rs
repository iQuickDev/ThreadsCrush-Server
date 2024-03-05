// put this in src/common/settings.rs
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Settings {
    pub page_size: u64,
}

impl Settings {
    pub fn from_json(value: &serde_json::Value) -> Result<Self> {
        Ok(serde_json::from_value(value.clone())?)
    }
}
