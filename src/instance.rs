/// This is a minecraft instance
///
/// # Example
/// ```
/// use minecraft_auth::instance::Instance;
///
/// let instance = Instance::default();
/// // or
/// let instance = Instance::new("1.18".to_string(), "/game/dir".to_string());
/// ```
pub struct Instance {
    /// This is the name of the folder content
    /// all minecraft file of this instance
    pub name: String,

    /// The game dir is a formatted path to all
    /// minecraft instance download and the current name
    pub game_dir: String,

    /// Minecraft assets directory
    pub assets_dir: String,
    /// Minecraft asset index
    /// (ex: 1.12)
    pub asset_index: String,

    /// Minecraft version
    /// (ex: 1.12.2)
    pub version: String,
    /// (ex: forge - release)
    pub version_type: String,

    /// Path of natives folder
    pub lib_path: String,
}

impl Default for Instance {
    // Create a default Instance
    fn default() -> Self {
        Self {
            name: "Default Instance".to_owned(),
            game_dir: "".to_owned(),
            assets_dir: "".to_owned(),
            lib_path: "".to_owned(),
            asset_index: "1.18".to_owned(),
            version: "1.18".to_owned(),
            version_type: "vanilla".to_owned(),
        }
    }
}

impl Instance {
    /// Create a new instance
    ///
    /// # Example
    /// ```
    /// use minecraft_auth::instance::Instance;
    ///
    /// let instance = Instance::new(
    ///         "Forge 1.12".to_owned(), "/game/dir".to_owned(),
    ///         "1.12.2-forge".to_owned(), "1.12".to_owned(), "Forge".to_owned());
    /// ```
    pub fn new(
        name: String,
        game_dir: String,
        version: String,
        asset_index: String,
        version_type: String,
    ) -> Self {
        Self {
            name,
            game_dir: game_dir.clone(),
            assets_dir: format!("{}/assets", game_dir),
            lib_path: format!("{}/natives", game_dir),
            asset_index,
            version,
            version_type,
        }
    }
}
