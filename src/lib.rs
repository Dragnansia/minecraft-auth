pub mod downloader;
pub mod instance;
pub mod user;
pub mod version;
pub mod native;

#[derive(Clone, Debug)]
pub struct MinecraftAuth {
    pub name: String,
    pub path: String,
    pub used_native: bool,
}

impl MinecraftAuth {
    pub fn new(name: String, path: String, used_native: bool) -> Self {
        Self {
            name,
            path,
            used_native,
        }
    }

    /// Create MinecraftAuth with just a name, and get
    /// os data dir to create a new folder
    pub fn new_just_name(name: String, used_native: bool) -> Option<Self> {
        match dirs::data_dir() {
            Some(d) => {
                let pp = d.as_path().to_str().unwrap();
                let path = format!("{}/{}", pp, name);
                std::fs::create_dir_all(path.clone()).unwrap();
                Some(Self {
                    name,
                    path,
                    used_native,
                })
            }
            None => None,
        }
    }
}
