use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub t: String,
    pub url: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ManifestVersion {
    pub latest: Latest,
    pub versions: Vec<Version>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}
