use crate::{
    data::{download::Classifier, package::Package},
    downloader::FileInfo,
    error::{self, Error},
    java::find_java_version,
    native::os_native_name,
    user::User,
    utils::scan,
    version::manifest,
    MinecraftAuth,
};
use log::{error, info};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::{
    collections::HashMap,
    env,
    fmt::{self, Display, Formatter},
    fs::{self, create_dir_all, File},
    io::{self, BufRead, BufReader, Write},
    path::Path,
    process::{Child, Command},
};
use zip::ZipArchive;

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

    pub fn as_int(&self) -> Option<i32> {
        if let DataParam::Int(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<String> {
        if let DataParam::Str(val) = self {
            Some(val.clone())
        } else {
            None
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

#[derive(Debug, Copy, Clone)]
pub struct Config {
    pub ram_max: i32,
    pub ram_min: i32,

    pub window_width: i32,
    pub window_height: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ram_max: 1024,
            ram_min: 512,
            window_height: 600,
            window_width: 800,
        }
    }
}

impl Config {
    pub fn new(ram_min: i32, ram_max: i32, window_width: i32, window_height: i32) -> Self {
        Self {
            ram_max,
            ram_min,
            window_width,
            window_height,
        }
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
        config: Config,
    ) -> Result<Self, error::Error> {
        let path = format!("{}/instances/{}", app.path, &name);
        let config_file_path = format!("{}/config.cfg", path);
        if Path::new(&config_file_path).exists() {
            Instance::from_config(app, name)
        } else if create_dir_all(&path).is_ok() {
            let mut param = HashMap::new();

            param.insert(
                "version".into(),
                Param::new(DataParam::Str(version.to_string()), true),
            );
            param.insert(
                "ramMin".into(),
                Param::new(DataParam::Int(config.ram_min), true),
            );
            param.insert(
                "ramMax".into(),
                Param::new(DataParam::Int(config.ram_max), true),
            );
            param.insert(
                "windowWidth".into(),
                Param::new(DataParam::Int(config.window_width), true),
            );
            param.insert(
                "windowHeight".into(),
                Param::new(DataParam::Int(config.window_height), true),
            );

            let mut this = Self {
                is_new: true,
                param,
            };

            let manifest = manifest(app, version)?;

            this.end_init_instance(app, &manifest, name, version)?;
            Ok(this)
        } else {
            Err(InstanceCreateError::FolderCreateError.into())
        }
    }

    fn end_init_instance(
        &mut self,
        app: &MinecraftAuth,
        manifest: &Package,
        name: &str,
        version: &str,
    ) -> Result<(), Error> {
        let path = format!("{}/instances/{}", app.path, name);

        self.add_param("name", Param::new(DataParam::Str(name.to_string()), false));
        self.add_param("path", Param::new(DataParam::Str(path.clone()), false));

        self.add_param(
            "libs",
            Param::new(
                DataParam::Str(get_all_libs_of_version(app, version)?),
                false,
            ),
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
                DataParam::Int(manifest.java_version.major_version as i32),
                false,
            ),
        );
        self.add_param(
            "assetIndex",
            Param::new(DataParam::Str(manifest.assets.clone()), false),
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
                Param::new(DataParam::Str(manifest.t.clone()), false),
            );
            self.add_param(
                "mainClass",
                Param::new(DataParam::Str(manifest.main_class.clone()), false),
            );
        }

        install_natives_file(app, &path, manifest)?;

        if self.is_new {
            self.save_config()?;
        }

        Ok(())
    }

    // Need to find a method to download forge file
    pub fn install_forge(&mut self) -> Result<(), error::Error> {
        self.add_param("useForge", Param::new(DataParam::Str("true".into()), true));
        self.save_config()?;

        self.update_param("versionType", DataParam::Str("Forge".to_string()));
        self.update_param(
            "mainClass",
            DataParam::Str("net.minecraft.launchwrapper.Launch".to_string()),
        );
        self.add_param(
            "tweakClass",
            Param::new(
                DataParam::Str("net.minecraftforge.fml.common.launcher.FMLTweaker".to_string()),
                false,
            ),
        );

        Ok(())
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

    pub fn update_param(&mut self, name: &str, val: DataParam) -> Option<()> {
        let v = self.param.get_mut(name)?;
        v.data = val;

        Some(())
    }

    /// Return vec with all arguments for start instance
    pub fn args(&self, app: &MinecraftAuth, user: &User) -> Vec<String> {
        let mut v = vec![
            #[cfg(target_os = "windows")]
            "-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump".into(),
            format!("-Xms{}m", self.param("ramMin")),
            format!("-Xmx{}m", self.param("ramMax")),
            format!("-Djava.library.path={}", self.param("gameDir")),
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

    pub fn save_config(&self) -> Result<(), error::Error> {
        let p = format!("{}/config.cfg", self.param("path"));
        let mut file = File::create(p)?;
        file.write_all(self.config_to_string().as_bytes())?;

        Ok(())
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
    pub fn from_config(app: &MinecraftAuth, name: &str) -> Result<Self, error::Error> {
        let p = format!("{}/instances/{}/config.cfg", app.path, name);
        let file = File::open(&p)?;
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

        let version = this
            .param("version")
            .as_string()
            .ok_or(InstanceCreateError::NoFoundVersion)?;
        let manifest = manifest(app, &version)?;
        this.end_init_instance(app, &manifest, name, &version)?;
        Ok(this)
    }
}

/// Install natives files on `{instance_path}/natives`
fn install_natives_file(
    app: &MinecraftAuth,
    instance_path: &str,
    manifest: &Package,
) -> Result<(), error::Error> {
    let native_dir = format!("{}/natives", instance_path);
    fs::create_dir_all(&native_dir)?;

    let os_name = os_native_name();
    for libs in &manifest.libraries {
        let classifiers = &libs.downloads.classifiers;
        if classifiers.is_none() {
            continue;
        }

        if let Some(Classifier::Complex(data)) = classifiers {
            if !data.contains_key(os_name) {
                continue;
            }

            let file_path = format!(
                "{}/libraries/{}",
                app.path,
                data[os_name].path.clone().unwrap()
            );

            let file = File::open(file_path)?;
            let mut zip = ZipArchive::new(file)?;
            zip.extract(native_dir.clone())?;
        }
    }

    Ok(())
}

pub fn get_all_libs_of_version(app: &MinecraftAuth, version: &str) -> Result<String, Error> {
    let mut libs = String::new();
    let s = if cfg!(windows) { ';' } else { ':' };

    let manifest: Package = manifest(app, version)?;
    for lib in manifest.libraries {
        let l = if let Some(artifact) = &lib.downloads.artifact {
            artifact.path.clone().unwrap_or_default()
        } else if let Some(Classifier::Simple(classifiers)) = &lib.downloads.classifiers {
            classifiers.path.clone().unwrap_or_default()
        } else {
            "".into()
        };

        libs += &format!("{}/libraries/{}{}", app.path, l, s);
    }

    libs += &format!("{}/clients/{}/client.jar", app.path, version);
    Ok(libs)
}

fn change_current_dir<P: AsRef<Path>>(dir: P) -> io::Result<()> {
    env::set_current_dir(dir)
}

fn java_is_command() -> bool {
    Command::new("java").arg("-h").output().is_ok()
}

// Find better java version for version
/// Start minecraft instance and return a child process
pub fn start_instance(
    app: &MinecraftAuth,
    user: &User,
    i: &Instance,
) -> Result<Child, error::Error> {
    if let DataParam::Int(version) = i.param("javaVersion") {
        let current_dir = env::current_dir()?;

        change_current_dir(i.param("gameDir").to_string())?;
        let java_command = if let Some(java) = find_java_version(version as u8) {
            java
        } else {
            error!("No found java version {}", version);
            info!("Try to use java command instead");

            if !java_is_command() {
                return Err(format!(
                    "java command is not found, please reinstall java {}",
                    version
                )
                .into());
            }

            String::from("java")
        };

        let mut cmd = Command::new(java_command);
        cmd.args(i.args(app, user));

        #[cfg(windows)]
        // No open console windows when spawn command
        cmd.creation_flags(0x08000000);

        let process = cmd.spawn()?;
        change_current_dir(current_dir)?;
        Ok(process)
    } else {
        Err("No found javaVersion param on Instance".into())
    }
}
