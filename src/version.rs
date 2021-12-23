use std::{fs::read_to_string, path::Path};

use crate::{
    downloader::{download_file, RefDownloader},
    MinecraftAuth,
};
use futures::future::join3;
use serde_json::Value;

pub fn manifest(app: &MinecraftAuth, version: &str) -> Option<Value> {
    let p = format!("{}/versions/{}.json", app.path, version);
    let path = Path::new(&p);
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

pub fn version_manifest(app: &MinecraftAuth, version: &str) -> Option<Value> {
    let p = format!("{}/assets/indexes/{}.json", app.path, version);
    let path = Path::new(&p);
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

async fn download_manifest(app: &MinecraftAuth, url: &str, id: &str) -> Result<String, String> {
    download_file(
        url.to_string(),
        format!("{}/versions/{}.json", app.path, id),
        None,
    )
    .await
}

async fn download_manifest_version(
    app: &MinecraftAuth,
    url: &str,
    id: &str,
) -> Result<String, String> {
    download_file(
        url.to_string(),
        format!("{}/assets/indexes/{}.json", app.path, id),
        None,
    )
    .await
}

// async fn intern_download_version(app: &MinecraftAuth, _: Vec<(&str, &str)>) {}

#[cfg(target_os = "linux")]
fn os_native_name() -> &'static str {
    "natives-linux"
}

#[cfg(target_os = "windows")]
fn os_native_name() -> &'static str {
    "natives-windows"
}

#[cfg(target_os = "macos")]
fn os_native_name() -> &'static str {
    "natives-osx"
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

fn add_download_with_lib_info(infos: &Value, lib_path: &str, downloader: &RefDownloader) {
    let url = infos["url"].as_str().unwrap().to_string();
    let path = format!("{}{}", lib_path, infos["path"].as_str().unwrap());
    downloader
        .lock()
        .unwrap()
        .add_download(url, path.clone(), path);
}

// Find a way to return a downloader to user with all download file
async fn download_libraries(
    app: &MinecraftAuth,
    libs: Option<&Vec<Value>>,
    downloader: &RefDownloader,
) {
    let lib_path = format!("{}/libraries/", app.path);

    if let Some(array) = libs {
        if app.used_native {
            array.iter().for_each(|a| {
                if let Some(artifact) = get_artifact(&a) {
                    add_download_with_lib_info(artifact, &lib_path, downloader);
                } else if let Some(classifiers) = get_classifiers(&a) {
                    add_download_with_lib_info(classifiers, &lib_path, downloader);
                } else {
                    println!(
                        "[Error] can't find artifact or classifiers on section for {:?}",
                        a["name"]
                    );
                }
            });
        } else {
            // Update this to download just os specific file
            array.iter().for_each(|a| {
                if let Some(artifact) = get_classifiers(&a) {
                    add_download_with_lib_info(artifact, &lib_path, &downloader);
                } else if let Some(classifiers) = get_artifact(&a) {
                    add_download_with_lib_info(classifiers, &lib_path, &downloader);
                } else {
                    println!(
                        "[Error] can't find artifact or classifiers on section for {:?}",
                        a["name"]
                    );
                }
            });
        }
    }
}

async fn download_client(
    app: &MinecraftAuth,
    client: &Value,
    downloader: &RefDownloader,
    version: &str,
) {
    let path = format!("{}/clients/{}/client.jar", app.path, version);
    downloader.lock().unwrap().add_download(
        client["url"].as_str().unwrap().to_string(),
        path.clone(),
        path,
    );
}

async fn download_assets(app: &MinecraftAuth, assets: &Value, downloader: &RefDownloader) {
    let id = assets["id"].as_str().unwrap();

    if let Ok(_) = download_manifest_version(app, assets["url"].as_str().unwrap(), &id).await {
        if let Some(manifest) = version_manifest(app, &id) {
            for m in manifest["objects"].as_object().unwrap() {
                let hash = m.1["hash"].as_str().unwrap();
                let f = &hash[..2];
                let path = format!("{}/assets/objects/{}/{}", app.path, f, hash);
                let url = format!("http://resources.download.minecraft.net/{}/{}", f, hash);

                downloader
                    .lock()
                    .unwrap()
                    .add_download(url, path.clone(), path);
            }
        }
    }
}

async fn find_and_install_minecraft_version(
    app: &MinecraftAuth,
    version: &str,
    m: &Value,
    downloader: &RefDownloader,
) {
    if let Some(versions) = m["versions"].as_array() {
        for v in versions {
            let id = v["id"].as_str().unwrap();
            if version == id {
                if let Some(m) = manifest(app, id) {
                    let lib = download_libraries(app, m["libraries"].as_array(), downloader);
                    let client =
                        download_client(app, &m["downloads"]["client"], downloader, version);
                    let assets = download_assets(app, &m["assetIndex"], downloader);

                    join3(lib, client, assets).await;
                } else {
                    let v_manifest = v["url"].as_str().unwrap();
                    match download_manifest(app, v_manifest, id).await {
                        Ok(_) => {
                            let m = manifest(app, id).unwrap();
                            let lib =
                                download_libraries(app, m["libraries"].as_array(), downloader);
                            let client = download_client(
                                app,
                                &m["downloads"]["client"],
                                downloader,
                                version,
                            );
                            let assets = download_assets(app, &m["assetIndex"], downloader);

                            join3(lib, client, assets).await;
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
/// let downloader = Downloader::new_ref();
/// let app = MinecraftAuth::new_just_name("Launcher", true);
/// download_version(&app, "1.18.1", downloader.clone());
///
/// downloader.lock().unwrap().wait();
/// ```
pub async fn download_version(app: &MinecraftAuth, version: String, downloader: RefDownloader) {
    if let Some(manifest) = manifest(app, "manifest_version") {
        find_and_install_minecraft_version(app, &version, &manifest, &downloader).await;
    } else {
        match download_manifest(
            app,
            "https://launchermeta.mojang.com/mc/game/version_manifest.json",
            "manifest_version",
        )
        .await
        {
            Ok(_) => {
                if let Some(manifest) = manifest(app, "manifest_version") {
                    find_and_install_minecraft_version(app, &version, &manifest, &downloader).await;
                } else {
                    println!("[Error] Can't find manifest_version file");
                }
            }
            Err(err) => println!("[Error] {}", err),
        }
    }
}
