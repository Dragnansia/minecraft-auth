pub mod downloader;
pub mod instance;
pub mod user;
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
        match dirs::data_dir() {
            Some(d) => {
                let pp = d.as_path().to_str().unwrap();
                let path = format!("{}/{}", pp, name);
                std::fs::create_dir_all(path.clone()).unwrap();
                Some(Self { name, path })
            }
            None => None,
        }
    }
}

#[test]
fn create_minecraft_auth() {
    let app = MinecraftAuth::new_just_name("Launcher".to_owned()).unwrap();
    println!("{:?}", app);
}
