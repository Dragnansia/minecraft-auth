use crate::{
    downloader::FileInfo,
    java::find_java_version,
    native::os_native_name,
    user::User,
    utils::scan,
    version::{get_artifact, get_classifiers, manifest},
    MinecraftAuth,
};
use serde_json::Value;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs::{create_dir_all, File},
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Child, Command},
};
use zip::ZipArchive;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Debug)]
pub enum InstanceCreateError {
    NoFoundVersion,
    AlreadyExist,
    FolderCreateError,
    ReadConfigError(String),
    NoFoundManifestVersion,
    NeedDownload(Vec<FileInfo>),
}

#[derive(Debug, Clone)]
pub enum DataParam {
    Int(i32),
    Str(String),
    None,
}

impl DataParam {
    pub fn is_true(&self) -> bool {
        match self {
            DataParam::Int(i) => *i > 0,
            DataParam::Str(s) => s == "true",
            DataParam::None => false,
        }
    }
}

impl Display for DataParam {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let val = match self {
            DataParam::Int(i) => i.to_string(),
            DataParam::Str(s) => s.clone(),
            DataParam::None => "".into(),
        };

        write!(f, "{}", val)
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub data: DataParam,
    pub on_config: bool,
}

impl Param {
    pub fn new(data: DataParam, on_config: bool) -> Self {
        Self { data, on_config }
    }
}

impl Display for Param {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Instance {
    pub is_new: bool,
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
            if create_dir_all(&path).is_ok() {
                let mut param = HashMap::new();

                param.insert(
                    "version".into(),
                    Param::new(DataParam::Str(version.to_string()), true),
                );
                param.insert("ramMin".into(), Param::new(DataParam::Int(512), true));
                param.insert("ramMax".into(), Param::new(DataParam::Int(1024), true));
                param.insert("windowWidth".into(), Param::new(DataParam::Int(800), true));
                param.insert("windowHeight".into(), Param::new(DataParam::Int(600), true));

                let mut this = Self {
                    is_new: true,
                    param,
                };

                if let Some(manifest) = manifest(app, version) {
                    this.end_init_instance(app, &manifest, name, version);
                    Ok(this)
                } else {
                    Err(InstanceCreateError::NoFoundManifestVersion)
                }
            } else {
                Err(InstanceCreateError::FolderCreateError)
            }
        }
    }

    fn end_init_instance(
        &mut self,
        app: &MinecraftAuth,
        manifest: &Value,
        name: &str,
        version: &str,
    ) {
        let path = format!("{}/instances/{}", app.path, name);

        self.add_param("name", Param::new(DataParam::Str(name.to_string()), false));
        self.add_param("path", Param::new(DataParam::Str(path.clone()), false));

        self.add_param(
            "libs",
            Param::new(DataParam::Str(get_all_libs_of_version(app, version)), false),
        );
        self.add_param(
            "assetsDir",
            Param::new(DataParam::Str(format!("{}/assets", app.path)), false),
        );
        self.add_param(
            "gameDir",
            Param::new(DataParam::Str(format!("{}/.minecraft", path)), false),
        );
        self.add_param(
            "nativeDir",
            Param::new(DataParam::Str(format!("{}/natives", path)), false),
        );
        self.add_param(
            "javaVersion",
            Param::new(
                DataParam::Int(manifest["javaVersion"]["majorVersion"].as_i64().unwrap() as i32),
                false,
            ),
        );
        self.add_param(
            "assetIndex",
            Param::new(
                DataParam::Str(manifest["assets"].as_str().unwrap().to_string()),
                false,
            ),
        );

        if self.param("useForge").is_true() {
            self.add_param(
                "versionType",
                Param::new(DataParam::Str("Forge".to_string()), false),
            );
            self.add_param(
                "mainClass",
                Param::new(
                    DataParam::Str("net.minecraft.launchwrapper.Launch".to_string()),
                    false,
                ),
            );
            self.add_param(
                "tweakClass",
                Param::new(
                    DataParam::Str("net.minecraftforge.fml.common.launcher.FMLTweaker".to_string()),
                    false,
                ),
            );
        } else {
            self.add_param(
                "versionType",
                Param::new(
                    DataParam::Str(manifest["type"].as_str().unwrap().to_string()),
                    false,
                ),
            );
            self.add_param(
                "mainClass",
                Param::new(
                    DataParam::Str(manifest["mainClass"].as_str().unwrap().to_string()),
                    false,
                ),
            );
        }

        if self.is_new {
            install_natives_file(app, &path, &manifest);
            self.save_config();
        }
    }

    // Need to find a method to download forge file
    pub fn install_forge(&mut self) {
        self.add_param("useForge", Param::new(DataParam::Str("true".into()), false));
        self.save_config();

        self.update_param("versionType", DataParam::Str("Forge".to_string()));
        self.update_param(
            "mainClass",
            DataParam::Str("net.minecraft.launchwrapper.Launch".to_string()),
        );
        self.update_param(
            "tweakClass",
            DataParam::Str("net.minecraftforge.fml.common.launcher.FMLTweaker".to_string()),
        );
    }

    pub fn add_param(&mut self, name: &str, val: Param) -> Option<Param> {
        self.param.insert(name.to_string(), val)
    }

    pub fn param(&self, name: &str) -> DataParam {
        if let Some(val) = self.param.get(name) {
            val.data.clone()
        } else {
            DataParam::None
        }
    }

    pub fn update_param(&mut self, name: &str, val: DataParam) {
        if let Some(v) = self.param.get_mut(name) {
            v.data = val;
        }
    }

    /// Return vec with all arguments for start instance
    pub fn args(&self, app: &MinecraftAuth, user: &User) -> Vec<String> {
        let mut v = vec![
            #[cfg(target_os = "windows")]
            "-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump".into(),
            format!("-Xms{}m", self.param("ramMin")),
            format!("-Xmx{}m", self.param("ramMax")),
            format!("-Djava.library.path={}", self.param("nativeDir")),
            format!(
                "-Dorg.lwjgl.librarypath={}",
                self.param("nativeDir")
            ),
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
        ];

        if self.param("useForge").is_true() {
            v.append(&mut vec![
                "--tweakClass".into(),
                self.param("tweakClass").to_string(),
            ]);
        }

        v
    }

    pub fn save_config(&self) {
        let p = format!("{}/config.cfg", self.param("path"));
        match File::create(p) {
            Ok(mut file) => {
                file.write_all(self.config_to_string().as_bytes()).unwrap();
            }
            Err(err) => println!("[Error] {:?}", err),
        };
    }

    fn config_to_string(&self) -> String {
        let mut s = String::new();
        self.param.iter().for_each(|o| {
            s += &if o.1.on_config {
                format!("{}={}\n", o.0, o.1)
            } else {
                "".into()
            };
        });

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
                    param.insert(
                        name.unwrap(),
                        Param::new(DataParam::Str(val.unwrap()), true),
                    );
                });

                let mut this = Self {
                    is_new: false,
                    param,
                };

                if let DataParam::Str(version) = this.param("version") {
                    if let Some(manifest) = manifest(app, &version) {
                        this.end_init_instance(app, &manifest, name, &version);
                        Ok(this)
                    } else {
                        Err(InstanceCreateError::NoFoundManifestVersion)
                    }
                } else {
                    Err(InstanceCreateError::NoFoundVersion)
                }
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
pub fn start_instance(app: &MinecraftAuth, user: &User, i: &Instance) -> Result<Child, String> {
    if let DataParam::Int(version) = i.param("javaVersion") {
        if let Some(java) = find_java_version(version as u8) {
            match Command::new(java).args(i.args(app, user)).spawn() {
                Ok(process) => Ok(process),
                Err(err) => Err(err.to_string()),
            }
        } else {
            Err(format!("No found java version {}", version))
        }
    } else {
        Err("No found javaVersion param on Instance".to_string())
    }
}
