pub mod downloader;
pub mod instance;
pub mod user;

pub struct MinecraftAuth {
    pub app_name: String,
    pub app_path: String,
    pub instance_path: String,
    pub cache_path: String,
}

impl MinecraftAuth {
    pub fn new(
        app_name: String,
        app_path: String,
        instance_path: String,
        cache_path: String,
    ) -> Self {
        Self {
            app_name,
            app_path,
            instance_path,
            cache_path,
        }
    }
}
