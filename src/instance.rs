use crate::{
    downloader::Downloader,
    user::User,
    version::{download_version, get_artifact, get_classifiers, manifest, os_native_name},
    MinecraftAuth,
};
use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    io::{self, BufRead, BufReader, Write},
    path::Path,
    process::{Child, Command},
};
use subprocess::{Exec, Popen, PopenError};
use zip::ZipArchive;

macro_rules! scan {
    ( $string:expr, $sep:expr, $( $x:ty ),+ ) => {{
        let mut iter = $string.split($sep);
        ($(iter.next().and_then(|word| word.parse::<$x>().ok()),)*)
    }}
}

#[derive(Debug)]
pub enum InstanceCreateError {
    AlreadyExist,
    FolderCreateError,
    ReadConfigError,
}

#[derive(Debug, Default)]
pub struct Instance {
    /// This is the name of the folder content
    /// minecraft file of this instance
    pub name: String,
    pub path: String,

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
        path: String,
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
            path,
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

    pub fn update_config(&self) {
        let p = format!("{}/config.cfg", self.path);
        match File::create(p) {
            Ok(mut file) => {
                let _ = file.write_all(self.config_to_byte().as_bytes());
            }
            Err(err) => println!("[Error] {:?}", err),
        };
    }

    fn config_to_byte(&self) -> String {
        let mut s = format!("name={}\n", self.name);
        s += &format!("path={}\n", self.path);
        s += &format!("game_dir={}\n", self.game_dir);
        s += &format!("native_dir={}\n", self.native_dir);
        s += &format!("class_name={}\n", self.class_name);
        s += &format!("assets_dir={}\n", self.assets_dir);
        s += &format!("asset_index={}\n", self.asset_index);
        s += &format!("version={}\n", self.version);
        s += &format!("version_type={}\n", self.version_type);
        s += &format!("libs={}\n", self.libs);
        s += &format!("ram_min={}\n", self.ram_min);
        s += &format!("ram_max={}\n", self.ram_max);
        s += &format!("window_width={}\n", self.window_width);
        s += &format!("window_height={}\n", self.window_height);
        s += &format!("current_language={}\n", self.current_language);
        s
    }

    pub fn from_config(app: &MinecraftAuth, name: &str) -> Result<Self, InstanceCreateError> {
        let p = format!("{}/instances/{}/config.cfg", app.path, name);
        match File::open(&p) {
            Ok(file) => {
                let buffer = BufReader::new(file);
                let mut h = HashMap::new();
                buffer.lines().for_each(|line| {
                    let l = line.unwrap();
                    let (name, val) = scan!(l, '=', String, String);
                    h.insert(name.unwrap(), val.unwrap());
                });

                Ok(Self {
                    name: h["name"].clone(),
                    path: h["path"].clone(),
                    game_dir: h["game_dir"].clone(),
                    native_dir: h["native_dir"].clone(),
                    class_name: h["class_name"].clone(),
                    assets_dir: h["assets_dir"].clone(),
                    asset_index: h["asset_index"].clone(),
                    version: h["version"].clone(),
                    version_type: h["version_type"].clone(),
                    libs: h["libs"].clone(),
                    ram_min: h["ram_min"].parse::<i32>().unwrap(),
                    ram_max: h["ram_max"].parse::<i32>().unwrap(),
                    window_width: h["window_width"].parse::<i32>().unwrap(),
                    window_height: h["window_height"].parse::<i32>().unwrap(),
                    current_language: h["current_language"].clone(),
                })
            }
            Err(_) => Err(InstanceCreateError::ReadConfigError),
        }
    }

    pub async fn create_new_instance(
        app: &MinecraftAuth,
        name: &str,
        version: &str,
    ) -> Result<Self, InstanceCreateError> {
        let path = format!("{}/instances/{}", app.path, &name);
        if Path::new(&path).exists() {
            Instance::from_config(app, name)
        } else {
            if let Ok(_) = create_dir_all(&path) {
                let downloader = Downloader::new_ref();
                download_version(app, version.to_string(), downloader.clone()).await;
                downloader.lock().unwrap().wait();

                install_natives_file(app, &path, version);

                let new_instance = Self {
                    name: name.to_string(),
                    path: path.clone(),
                    game_dir: format!("{}/.minecraft", path),
                    native_dir: format!("{}/natives", path),
                    class_name: "net.minecraft.client.main.Main".to_string(),
                    assets_dir: format!("{}/assets", app.path),
                    asset_index: "1.12".to_string(),
                    version: version.to_string(),
                    version_type: "vanilla".to_string(),
                    libs: get_all_libs_of_version(app, version.clone()),
                    ram_min: 512,
                    ram_max: 1024,
                    window_width: 800,
                    window_height: 600,
                    current_language: "en".to_string(),
                };
                new_instance.update_config();

                Ok(new_instance)
            } else {
                Err(InstanceCreateError::FolderCreateError)
            }
        }
    }
}

fn install_natives_file(app: &MinecraftAuth, instance_path: &str, version: &str) {
    if let Some(manifest) = manifest(app, version) {
        let native_dir = format!("{}/natives", instance_path);
        for libs in manifest["libraries"].as_array().unwrap() {
            let classifiers = &libs["downloads"]["classifiers"];
            if !classifiers.is_null() {
                let native = &classifiers[os_native_name()];
                if native.is_null() {
                    continue;
                }

                let file_path = format!(
                    "{}/libraries/{}",
                    app.path,
                    native["path"].as_str().unwrap()
                );
                match File::open(file_path) {
                    Ok(file) => {
                        let mut zip = ZipArchive::new(file).unwrap();
                        let _ = zip.extract(native_dir.clone());
                    }
                    Err(err) => {
                        println!("[Error] {}", err)
                    }
                }
            }
        }
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

    Exec::shell(cmd).popen()
}

/// Try to used this
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
