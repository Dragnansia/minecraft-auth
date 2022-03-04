use super::download::Download;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Library {
    pub downloads: Download,
    pub name: String,
}
