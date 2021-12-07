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
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            name: "Default Instance".to_owned(),
            game_dir: "".to_owned(),
        }
    }
}

impl Instance {
    pub fn new(name: String, game_dir: String) -> Self {
        Self { name, game_dir }
    }
}
