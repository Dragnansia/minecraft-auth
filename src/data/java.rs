use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: i32,
}
