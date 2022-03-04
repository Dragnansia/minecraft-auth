use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Download {
    pub artifact: Option<Artifact>,
    pub classifiers: Option<Artifact>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Artifact {
    pub path: Option<String>,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}
