use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Asset {
    pub hash: String,
    pub size: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    pub url: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Assets {
    pub objects: HashMap<String, Asset>,
}
