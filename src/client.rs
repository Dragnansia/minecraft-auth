use crate::{instance::Instance, user::User};
use std::process::{Child, Command};

#[derive(Debug)]
pub struct Client {
    pub ram_min: i32,
    pub ram_max: i32,
    pub window_width: i32,
    pub window_height: i32,
    pub current_language: String,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            ram_min: 512,
            ram_max: 1024,
            window_width: 854,
            window_height: 480,
            current_language: "en".to_owned(),
        }
    }
}

impl Client {
    pub fn new(
        ram_min: i32,
        ram_max: i32,
        window_width: i32,
        window_height: i32,
        current_language: String,
    ) -> Self {
        Self {
            ram_min,
            ram_max,
            window_height,
            window_width,
            current_language,
        }
    }

    /// Run instance on child process and
    /// returning a handle to it
    ///
    /// # Example
    /// ```
    /// use minecraft_auth::{instance::Instance, client::Client, user::User};
    ///
    /// let client = Client::default();
    /// let instance = Instance::default();
    /// let user = User::default();
    /// client.start_instance(&instance, &user);
    /// ```
    pub fn start_instance(&self, instance: &Instance, user: &User) -> Child {
        Command::new("java")
            .args([
                format!("-Xms{}M", self.ram_min),
                format!("-Xmx{}M", self.ram_max),
                format!("-Duser.language={}", self.current_language),
                format!("-Djava.library.path={}", instance.lib_path),
                "--version".to_owned(),
                instance.version.clone(),
                "--versionType".to_owned(),
                instance.version_type.clone(),
                "--gameDir".to_owned(),
                instance.game_dir.clone(),
                "--assetsDir".to_owned(),
                instance.assets_dir.clone(),
                "--assetIndex".to_owned(),
                instance.asset_index.clone(),
                "--username".to_owned(),
                user.username.clone(),
                "--accessToken".to_owned(),
                user.access_token.clone(),
                "--userType".to_owned(),
                "mojang".to_owned(),
                "--width".to_owned(),
                self.window_width.to_string(),
                "--height".to_owned(),
                self.window_height.to_string(),
            ])
            .spawn()
            .expect("Failed to launch Minecraft")
    }
}
