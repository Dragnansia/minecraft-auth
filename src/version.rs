use crate::{
    downloader::{download_file, FileInfo},
    error,
    native::os_native_name,
    MinecraftAuth,
};
use serde_json::Value;
use std::{
    fs::{read_to_string, File},
    path::Path,
};

fn intern_manifest(p: &str) -> Option<Value> {
    let path = Path::new(p);
    if path.exists() && path.is_file() {
        let file_content = read_to_string(path).ok()?;
        serde_json::from_str(&file_content).ok()
    } else {
        None
    }
}

pub fn manifest(app: &MinecraftAuth, version: &str) -> Option<Value> {
    intern_manifest(&format!("{}/versions/{}.json", app.path, version))
}

pub fn version_manifest(app: &MinecraftAuth, version: &str) -> Option<Value> {
    intern_manifest(&format!("{}/assets/indexes/{}.json", app.path, version))
}

async fn download_manifest(path: &str, url: &str, id: &str) -> Result<(), error::Error> {
    download_file(url.to_string(), format!("{}/{}.json", path, id)).await
}

pub fn get_classifiers(val: &Value) -> Option<&Value> {
    let classifiers = &val["downloads"]["classifiers"][os_native_name()];
    if classifiers.is_null() {
        None
    } else {
        Some(classifiers)
    }
}

pub fn get_artifact(val: &Value) -> Option<&Value> {
    let artifact = &val["downloads"]["artifact"];
    if artifact.is_null() {
        None
    } else {
        Some(artifact)
    }
}

fn add_download_with_lib_info(
    infos: &Value,
    lib_path: &str,
    files: &mut Vec<FileInfo>,
) -> Option<()> {
    let url = infos["url"].as_str()?;
    let path = format!("{}{}", lib_path, infos["path"].as_str()?);
    let size = infos["size"].as_u64()?;

    let p = Path::new(&path);
    let file = File::open(&path).ok()?;

    if !p.exists() || file.metadata().ok()?.len() != size {
        files.push(FileInfo::new(url.to_string(), path, size));
    }

    Some(())
}

// Find a way to return a downloader to user with all download file
async fn download_libraries(
    app: &MinecraftAuth,
    libs: Option<&Vec<Value>>,
    files: &mut Vec<FileInfo>,
) -> Option<()> {
    let lib_path = format!("{}/libraries/", app.path);
    let array = libs?;

    array.iter().for_each(|a| {
        if let Some(artifact) = get_artifact(a) {
            add_download_with_lib_info(artifact, &lib_path, files);
        }

        if let Some(classifiers) = get_classifiers(a) {
            add_download_with_lib_info(classifiers, &lib_path, files);
        }
    });

    Some(())
}

async fn download_client(
    app: &MinecraftAuth,
    client: &Value,
    version: &str,
    files: &mut Vec<FileInfo>,
) -> Option<()> {
    let path = format!("{}/clients/{}/client.jar", app.path, version);
    let size = client["size"].as_u64()?;

    let file = File::open(&path).ok()?;
    let p = Path::new(&path);

    if !p.exists() || file.metadata().ok()?.len() != size {
        files.push(FileInfo::new(client["url"].as_str()?.into(), path, size));
    }

    Some(())
}

async fn download_assets(
    app: &MinecraftAuth,
    assets: &Value,
    files: &mut Vec<FileInfo>,
) -> Option<()> {
    let id = assets["id"].as_str()?;
    let url = assets["url"].as_str()?;
    let path = format!("{}/assets", app.path);
    let indexes_path = format!("{}/indexes/", path);

    download_manifest(&indexes_path, url, id).await.ok()?;

    let manifest = version_manifest(app, id)?;
    files.append(
        &mut manifest["objects"]
            .as_object()?
            .iter()
            .filter_map(|object| {
                let hash = object.1["hash"].as_str()?;
                let b = &hash[..2];
                let p = format!("{}/objects/{}/{}", path, b, hash);
                let url = format!("http://resources.download.minecraft.net/{}/{}", b, hash);
                let size = object.1["size"].as_u64()?;

                let path = Path::new(&p);
                let file = File::open(&p).ok()?;

                if path.exists() || file.metadata().ok()?.len() != size {
                    return None;
                }

                Some(FileInfo::new(url, p, size))
            })
            .collect(),
    );

    Some(())
}

async fn find_and_install_minecraft_version(
    app: &MinecraftAuth,
    version: &str,
    m: &Value,
    files: &mut Vec<FileInfo>,
) -> Result<(), error::Error> {
    let versions = m["versions"]
        .as_array()
        .ok_or("Can't find versions on manifest json")?;

    let v = versions
        .iter()
        .find(|v| {
            let id = v["id"].as_str().unwrap();
            version == id
        })
        .ok_or("No version found")?;
    let id = v["id"].as_str().ok_or("Error to convert ID")?;

    if let Some(m) = manifest(app, id) {
        download_libraries(app, m["libraries"].as_array(), files).await;
        download_client(app, &m["downloads"]["client"], version, files).await;
        download_assets(app, &m["assetIndex"], files).await;
    } else {
        let path = format!("{}/versions/", app.path);
        let url = v["url"].as_str().ok_or("Error to convert URL")?;
        download_manifest(&path, url, id).await?;

        let m = manifest(app, id).ok_or("No manifest")?;
        download_libraries(app, m["libraries"].as_array(), files).await;
        download_client(app, &m["downloads"]["client"], version, files).await;
        download_assets(app, &m["assetIndex"], files).await;
    }

    Ok(())
}

/// Used to add all file to download on a Downloader
/// and user can just wait and get status of the current file downloader
///
/// # Examples
/// ```
/// ```
pub async fn file_to_download_for_version(
    app: &MinecraftAuth,
    version: String,
) -> Result<Vec<FileInfo>, error::Error> {
    let mut files = vec![];

    if let Some(manifest) = manifest(app, "manifest_version") {
        find_and_install_minecraft_version(app, &version, &manifest, &mut files).await?;
    } else {
        let path = format!("{}/versions", app.path);
        download_manifest(
            &path,
            "https://launchermeta.mojang.com/mc/game/version_manifest.json",
            "manifest_version",
        )
        .await?;

        let manifest =
            manifest(app, "manifest_version").ok_or("Can't find manifest_version file")?;
        find_and_install_minecraft_version(app, &version, &manifest, &mut files).await?
    }

    Ok(files)
}
