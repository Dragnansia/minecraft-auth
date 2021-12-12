use std::process::{Child, Command};

#[derive(Debug, Default)]
pub struct Instance {
    /// This is the name of the folder content
    /// minecraft file of this instance
    pub name: String,

    /// The game dir is a formatted path to the current
    /// minecraft instance
    pub game_dir: String,

    /// Minecraft assets directory
    pub assets_dir: String,
    /// Minecraft asset index
    pub asset_index: String,

    /// Minecraft version
    pub version: String,
    /// (ex: forge - release)
    pub version_type: String,

    /// Libraries path folder
    pub lib_path: String,

    /// Ram
    pub ram_min: i32,
    pub ram_max: i32,

    /// Window size
    pub window_width: i32,
    pub window_height: i32,

    /// Current language
    pub current_language: String,
}

impl Instance {
    pub fn new(
        name: String,
        game_dir: String,
        version: String,
        asset_index: String,
        version_type: String,
        ram_min: i32,
        ram_max: i32,
        window_width: i32,
        window_height: i32,
        current_language: String,
    ) -> Self {
        Self {
            name,
            game_dir: game_dir.clone(),
            assets_dir: format!("{}/assets", game_dir),
            lib_path: format!("{}/natives", game_dir),
            asset_index,
            version,
            version_type,
            ram_min,
            ram_max,
            window_width,
            window_height,
            current_language,
        }
    }
}

pub fn start_instance(_: &Instance) -> Option<Child> {
    let mut cmd = Command::new("java");
    let spawn = cmd.arg("");

    match spawn.spawn() {
        Ok(child) => Some(child),
        Err(_) => None,
    }
}
