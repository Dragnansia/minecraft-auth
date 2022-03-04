use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone)]
pub struct Download {
    pub artifact: Option<Artifact>,
    pub classifiers: Option<Classifier>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Artifact {
    pub path: Option<String>,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Classifier {
    Simple(Artifact),
    Complex(HashMap<String, Artifact>),
}
