use crate::{
    downloader::{download_file, FileInfo},
    native::os_native_name,
    MinecraftAuth,
};
use serde_json::Value;
use std::{fs::read_to_string, path::Path};

fn intern_manifest(p: &str) -> Option<Value> {
    let path = Path::new(p);
    if path.exists() && path.is_file() {
        if let Ok(file_content) = read_to_string(path) {
            Some(serde_json::from_str(&file_content).unwrap())
        } else {
            None
        }
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

async fn download_manifest(path: &str, url: &str, id: &str) -> Result<(), String> {
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

fn add_download_with_lib_info(infos: &Value, lib_path: &str, files: &mut Vec<FileInfo>) {
    let url = infos["url"].as_str().unwrap();
    let path = format!("{}{}", lib_path, infos["path"].as_str().unwrap());
    let size = infos["size"].as_u64().unwrap();
    let file = Path::new(&path);
    if !file.exists() {
        files.push(FileInfo::new(url.to_string(), path, size));
    }
}

// Find a way to return a downloader to user with all download file
async fn download_libraries(
    app: &MinecraftAuth,
    libs: Option<&Vec<Value>>,
    files: &mut Vec<FileInfo>,
) {
    let lib_path = format!("{}/libraries/", app.path);

    if let Some(array) = libs {
        array.iter().for_each(|a| {
            if let Some(artifact) = get_artifact(&a) {
                add_download_with_lib_info(artifact, &lib_path, files);
            }

            if let Some(classifiers) = get_classifiers(&a) {
                add_download_with_lib_info(classifiers, &lib_path, files);
            }
        });
    }
}

async fn download_client(
    app: &MinecraftAuth,
    client: &Value,
    version: &str,
    files: &mut Vec<FileInfo>,
) {
    let path = format!("{}/clients/{}/client.jar", app.path, version);
    if !Path::new(&path).exists() {
        files.push(FileInfo::new(
            client["url"].as_str().unwrap().to_string(),
            path,
            client["size"].as_u64().unwrap(),
        ));
    }
}

async fn download_assets(app: &MinecraftAuth, assets: &Value, files: &mut Vec<FileInfo>) {
    let id = assets["id"].as_str().unwrap();

    let path = format!("{}/assets", app.path);
    if let Ok(_) = download_manifest(
        &format!("{}/indexes/", path),
        assets["url"].as_str().unwrap(),
        &id,
    )
    .await
    {
        if let Some(manifest) = version_manifest(app, &id) {
            for m in manifest["objects"].as_object().unwrap() {
                let hash = m.1["hash"].as_str().unwrap();
                let f = &hash[..2];
                let p = format!("{}/objects/{}/{}", path, f, hash);
                let url = format!("http://resources.download.minecraft.net/{}/{}", f, hash);
                let size = m.1["size"].as_u64().unwrap();

                if !Path::new(&p).exists() {
                    files.push(FileInfo::new(url.to_string(), p, size));
                }
            }
        }
    }
}

async fn find_and_install_minecraft_version(
    app: &MinecraftAuth,
    version: &str,
    m: &Value,
    files: &mut Vec<FileInfo>,
) {
    if let Some(versions) = m["versions"].as_array() {
        for v in versions {
            let id = v["id"].as_str().unwrap();
            if version == id {
                if let Some(m) = manifest(app, id) {
                    download_libraries(app, m["libraries"].as_array(), files).await;
                    download_client(app, &m["downloads"]["client"], version, files).await;
                    download_assets(app, &m["assetIndex"], files).await;
                } else {
                    let path = format!("{}/versions/", app.path);
                    match download_manifest(&path, v["url"].as_str().unwrap(), id).await {
                        Ok(_) => {
                            let m = manifest(app, id).unwrap();
                            download_libraries(app, m["libraries"].as_array(), files).await;
                            download_client(app, &m["downloads"]["client"], version, files).await;
                            download_assets(app, &m["assetIndex"], files).await;
                        }
                        Err(err) => println!("[Error] {}", err),
                    }
                }

                break;
            }
        }
    } else {
        println!("Can't find versions on manifest json");
    }
}

/// Used to add all file to download on a Downloader
/// and user can just wait and get statut of the current file downloader
///
/// # Examples
/// ```
/// ```
pub async fn find_file_for_version(app: &MinecraftAuth, version: String) -> Vec<FileInfo> {
    let mut files = vec![];

    if let Some(manifest) = manifest(app, "manifest_version") {
        find_and_install_minecraft_version(app, &version, &manifest, &mut files).await;
    } else {
        let path = format!("{}/versions", app.path);
        match download_manifest(
            &path,
            "https://launchermeta.mojang.com/mc/game/version_manifest.json",
            "manifest_version",
        )
        .await
        {
            Ok(_) => {
                if let Some(manifest) = manifest(app, "manifest_version") {
                    find_and_install_minecraft_version(app, &version, &manifest, &mut files).await;
                } else {
                    println!("[Error] Can't find manifest_version file");
                }
            }
            Err(err) => println!("[Error] {}", err),
        }
    }

    files
}
