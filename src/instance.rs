use crate::{
    downloader::FileInfo,
    native::os_native_name,
    user::User,
    utils::scan,
    version::{get_artifact, get_classifiers, manifest},
    MinecraftAuth,
};
use serde_json::Value;
use std::{
    collections::HashMap,
    fmt::Display,
    fs::{create_dir_all, File},
    io::{self, BufRead, BufReader, Write},
    path::Path,
    process::{Child, Command},
};
use zip::ZipArchive;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;


#[derive(Debug)]
pub enum InstanceCreateError {
    AlreadyExist,
    FolderCreateError,
    ReadConfigError(String),
    NoFoundManifestVersion,
    NeedDownload(Vec<FileInfo>),
}

#[derive(Debug, Clone)]
pub enum Param {
    Int(i32),
    Str(String),
    None,
}

impl Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Param::Int(i) => i.to_string(),
            Param::Str(s) => s.clone(),
            Param::None => "".into(),
        };

        write!(f, "{}", val)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Instance {
    param: HashMap<String, Param>,
}

impl Instance {
    /// Create new instance
    pub async fn new(
        app: &MinecraftAuth,
        name: &str,
        version: &str,
    ) -> Result<Self, InstanceCreateError> {
        let path = format!("{}/instances/{}", app.path, &name);
        let config_file_path = format!("{}/config.cfg", path);
        if Path::new(&config_file_path).exists() {
            Instance::from_config(app, name)
        } else {
            if let Ok(_) = create_dir_all(&path) {
                if let Some(manifest) = manifest(app, version) {
                    install_natives_file(app, &path, &manifest);
                    let mut param = HashMap::new();

                    param.insert("name".into(), Param::Str(name.to_string()));
                    param.insert("path".into(), Param::Str(path.clone()));
                    param.insert("gameDir".into(), Param::Str(format!("{}/.minecraft", path)));
                    param.insert("nativeDir".into(), Param::Str(format!("{}/natives", path)));
                    param.insert(
                        "assetsDir".into(),
                        Param::Str(format!("{}/assets", app.path)),
                    );
                    param.insert(
                        "assetIndex".into(),
                        Param::Str(manifest["assets"].as_str().unwrap().to_string()),
                    );
                    param.insert(
                        "mainClass".into(),
                        Param::Str(manifest["mainClass"].as_str().unwrap().to_string()),
                    );
                    param.insert("version".into(), Param::Str(version.to_string()));
                    param.insert(
                        "versionType".into(),
                        Param::Str(manifest["type"].as_str().unwrap().to_string()),
                    );
                    param.insert(
                        "libs".into(),
                        Param::Str(get_all_libs_of_version(app, version.clone())),
                    );
                    param.insert("ramMin".into(), Param::Int(512));
                    param.insert("ramMax".into(), Param::Int(1024));
                    param.insert("windowWidth".into(), Param::Int(800));
                    param.insert("windowHeight".into(), Param::Int(600));

                    let new_instance = Self { param };
                    new_instance.update_config();

                    Ok(new_instance)
                } else {
                    Err(InstanceCreateError::NoFoundManifestVersion)
                }
            } else {
                Err(InstanceCreateError::FolderCreateError)
            }
        }
    }

    pub fn param(&self, name: &str) -> Param {
        if let Some(val) = self.param.get(name) {
            val.clone()
        } else {
            Param::None
        }
    }

    pub fn update_param(&mut self, name: &str, val: Param) {
        if let Some(v) = self.param.get_mut(name) {
            *v = val;
        }
    }

    pub fn add_param(&mut self, name: &str, val: Param) -> Option<Param> {
        self.param.insert(name.to_string(), val)
    }

    /// Return vec with all arguments for start instance
    pub fn args(&self, app: &MinecraftAuth, user: &User) -> Vec<String> {
        vec![
            format!("-Xms{}m", self.param("ramMin")),
            format!("-Xmx{}m", self.param("ramMax")),
            format!("-Djava.library.path={}", self.param("nativeDir")),
            format!("-Dorg.lwjgl.librarypath={}", self.param("nativeDir")),
            format!("-Dminecraft.launcher.brand={}", app.name),
            "-Dminecraft.launcher.version=2.1".to_string(),
            "-cp".to_string(),
            self.param("libs").to_string(),
            self.param("mainClass").to_string(),
            "--width".to_string(),
            self.param("windowWidth").to_string(),
            "--height".to_string(),
            self.param("windowHeight").to_string(),
            "--username".to_string(),
            user.username.clone(),
            "--accessToken".to_string(),
            user.access_token.clone(),
            "--uuid".to_string(),
            user.uuid.clone(),
            "--version".to_string(),
            self.param("version").to_string(),
            "--gameDir".to_string(),
            self.param("gameDir").to_string(),
            "--assetsDir".to_string(),
            self.param("assetsDir").to_string(),
            "--assetIndex".to_string(),
            self.param("assetIndex").to_string(),
        ]
    }

    pub fn update_config(&self) {
        let p = format!("{}/config.cfg", self.param("path"));
        match File::create(p) {
            Ok(mut file) => {
                let _ = file.write_all(self.config_to_string().as_bytes());
            }
            Err(err) => println!("[Error] {:?}", err),
        };
    }

    fn config_to_string(&self) -> String {
        let mut s = String::new();
        self.param
            .iter()
            .for_each(|o| s += &format!("{}={}\n", o.0, o.1));

        s
    }

    /// Load instance from config file
    pub fn from_config(app: &MinecraftAuth, name: &str) -> Result<Self, InstanceCreateError> {
        let p = format!("{}/instances/{}/config.cfg", app.path, name);
        match File::open(&p) {
            Ok(file) => {
                let buffer = BufReader::new(file);
                let mut param = HashMap::new();
                buffer.lines().for_each(|line| {
                    let l = line.unwrap();
                    let (name, val) = scan!(l, '=', String, String);
                    param.insert(name.unwrap(), Param::Str(val.unwrap()));
                });

                Ok(Self { param })
            }
            Err(err) => Err(InstanceCreateError::ReadConfigError(err.to_string())),
        }
    }
}

/// Install natives files on `{instance_path}/natives`
fn install_natives_file(app: &MinecraftAuth, instance_path: &str, manifest: &Value) {
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

pub fn get_all_libs_of_version(app: &MinecraftAuth, version: &str) -> String {
    let mut libs = String::new();
    let s = if cfg!(windows) { ';' } else { ':' };

    if let Some(manifest) = manifest(app, version) {
        for lib in manifest["libraries"].as_array().unwrap() {
            let l = if let Some(artifact) = get_artifact(lib) {
                artifact["path"].as_str().unwrap()
            } else if let Some(classifiers) = get_classifiers(lib) {
                classifiers["path"].as_str().unwrap()
            } else {
                ""
            };

            libs += &format!("{}/libraries/{}{}", app.path, l, s);
        }
    }

    libs += &format!("{}/clients/{}/client.jar", app.path, version);
    libs
}

// Find better java version for version
/// Start minecraft instance and return a child process
#[cfg(not(target_os = "windows"))]
pub fn start_instance(app: &MinecraftAuth, user: &User, i: &Instance) -> io::Result<Child> {
    let mut cmd = Command::new("java");
    i.args(app, user).iter().for_each(|el| {
        cmd.arg(el);
    });

    cmd.spawn()
}

#[cfg(target_os = "windows")]
pub fn start_instance(app: &MinecraftAuth, user: &User, i: &Instance) -> io::Result<Child> {
    let mut cmd = Command::new("java");
    i.args(app, user).iter().for_each(|el| {
        cmd.arg(el);
    });

    cmd.creation_flags(0x00000008);
    cmd.spawn()
}

pub fn start_forge_instance(app: &MinecraftAuth, user: &User, i: &Instance) -> io::Result<Child> {
    let mut cmd = Command::new("java");
    i.args(app, user).iter().for_each(|el| {
        cmd.arg(el);
    });
    cmd.arg("--tweakClass");
    cmd.arg(i.param("tweakClass").to_string());

    cmd.spawn()
}
