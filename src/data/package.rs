use super::{
    argument::Arguments, asset::AssetIndex, download::Artifact, java::JavaVersion, library::Library,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Package {
    pub arguments: Option<Arguments>,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    pub assets: String,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: i32,
    pub downloads: Downloads,
    pub id: String,
    #[serde(rename = "javaVersion")]
    pub java_version: JavaVersion,
    pub libraries: Vec<Library>,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "type")]
    pub t: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Downloads {
    pub client: Artifact,
    pub server: Artifact,
}
