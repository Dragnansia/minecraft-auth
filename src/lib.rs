use std::fs;

pub mod downloader;
pub mod error;
pub mod instance;
pub mod java;
pub mod native;
pub mod user;
pub mod utils;
pub mod version;

#[derive(Clone, Debug)]
pub struct MinecraftAuth {
    pub name: String,
    pub path: String,
}

impl MinecraftAuth {
    pub fn new(name: String, path: String) -> Self {
        Self { name, path }
    }

    /// Create MinecraftAuth with just a name, and get
    /// os data dir to create a new folder
    pub fn new_just_name(name: String) -> Option<Self> {
        let data_dir = dirs::data_dir()?;
        let temp_path = data_dir.as_path().to_str()?;
        let path = format!("{}/{}", temp_path, name);
        fs::create_dir_all(path.clone()).ok()?;

        Some(Self { name, path })
    }
}
