use crate::{
    user::User,
    version::{get_artifact, get_classifiers, manifest},
    MinecraftAuth,
};
use std::{
    io,
    process::{Child, Command},
};
use subprocess::{Exec, Popen, PopenError};

#[derive(Debug, Default)]
pub struct Instance {
    /// This is the name of the folder content
    /// minecraft file of this instance
    pub name: String,

    /// The game dir is a formatted path to the current
    /// minecraft instance
    pub game_dir: String,

    /// Natives dir for temp file
    pub native_dir: String,

    /// Main class name
    pub class_name: String,

    /// Minecraft assets directory
    pub assets_dir: String,
    /// Minecraft asset index
    pub asset_index: String,

    /// Minecraft version
    pub version: String,
    /// (ex: forge - release)
    pub version_type: String,

    /// Libraries path folder
    pub libs: String,

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
        assets_dir: String,
        libs: String,
        native_dir: String,
        class_name: String,
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
            game_dir,
            assets_dir,
            libs,
            native_dir,
            asset_index,
            class_name,
            version,
            version_type,
            ram_min,
            ram_max,
            window_width,
            window_height,
            current_language,
        }
    }

    pub fn args(&self, app: &MinecraftAuth, user: &User) -> Vec<String> {
        vec![
            format!("-Xms{}m", self.ram_min),
            format!("-Xmx{}m", self.ram_max),
            format!("-Djava.library.path={}", self.native_dir),
            format!("-Dorg.lwjgl.librarypath={}", self.native_dir),
            format!("-Dminecraft.launcher.brand={}", app.name),
            "-Dminecraft.launcher.version=2.1".to_string(),
            "-cp".to_string(),
            self.libs.clone(),
            self.class_name.clone(),
            "--width".to_string(),
            self.window_width.to_string(),
            "--height".to_string(),
            self.window_height.to_string(),
            "--username".to_string(),
            user.username.clone(),
            "--accessToken".to_string(),
            user.access_token.clone(),
            "--uuid".to_string(),
            user.uuid.clone(),
            "--version".to_string(),
            self.version.clone(),
            "--gameDir".to_string(),
            self.game_dir.clone(),
            "--assetsDir".to_string(),
            self.assets_dir.clone(),
            "--assetIndex".to_string(),
            self.asset_index.clone(),
        ]
    }
}

#[cfg(target_os = "linux")]
pub fn get_all_libs_of_version(app: &MinecraftAuth, version: &str) -> String {
    let mut libs = String::new();

    if let Some(manifest) = manifest(app, version) {
        for lib in manifest["libraries"].as_array().unwrap() {
            if let Some(artifact) = get_artifact(lib) {
                libs += &format!(
                    "\"{}/libraries/{}\":",
                    app.path,
                    artifact["path"].as_str().unwrap()
                );
            } else if let Some(classifiers) = get_classifiers(lib) {
                libs += &format!(
                    "\"{}/libraries/{}\":",
                    app.path,
                    classifiers["path"].as_str().unwrap()
                );
            }
        }
    }

    libs += &format!("\"{}/clients/{}/client.jar\"", app.path, version);
    libs
}

#[cfg(target_os = "macos")]
pub fn get_all_libs_of_version(app: &MinecraftAuth, version: &str) -> String {
    // find macos way
}

#[cfg(target_os = "windows")]
pub fn get_all_libs_of_version(app: &MinecraftAuth, version: &str) -> String {
    let mut libs = String::from("\"");

    if let Some(manifest) = version_manifest(app, version) {
        for lib in manifest["libraries"].as_array().unwrap() {
            if let Some(artifact) = get_artifact(lib) {
                libs += &format!(
                    "{}/libraries/{};",
                    app.path,
                    artifact["path"].as_str().unwrap()
                );
            } else if let Some(classifiers) = get_classifiers(lib) {
                libs += &format!(
                    "{}/libraries/{};",
                    app.path,
                    classifiers["path"].as_str().unwrap()
                );
            }
        }
    }

    libs += &format!("{}/clients/{}/client.jar;\"", app.path, version);
    libs
}

/// Not a secure approch of this
pub fn si(app: &MinecraftAuth, user: &User, i: &Instance) -> Result<Popen, PopenError> {
    let mut cmd =
        String::from("/usr/lib/jvm/java-1.8.0-openjdk-1.8.0.312.b07-2.fc35.x86_64/jre/bin/java ");

    for el in i.args(app, user) {
        cmd += &format!("{} ", el);
    }

    println!("\n\n\n\n{}\n\n\n", cmd);

    Exec::shell(cmd).popen()
}

pub fn start_instance(app: &MinecraftAuth, user: &User, i: &Instance) -> io::Result<Child> {
    let mut cmd =
        Command::new("/usr/lib/jvm/java-1.8.0-openjdk-1.8.0.312.b07-2.fc35.x86_64/jre/bin/java");
    let spawn = cmd.args(i.args(app, user));

    println!("\n\n");

    for x in spawn.get_args() {
        println!("{}", x.to_str().unwrap());
    }

    println!("\n\n");

    spawn.spawn()
}
