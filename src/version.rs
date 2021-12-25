use crate::{
    downloader::{download_file, RefDownloader},
    native::os_native_name,
    MinecraftAuth,
};
use futures::future::join3;
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

async fn download_manifest(path: &str, url: &str, id: &str) -> Result<String, String> {
    download_file(url.to_string(), format!("{}/{}.json", path, id), None).await
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
    let url = infos["url"].as_str().unwrap();
    let path = format!("{}{}", lib_path, infos["path"].as_str().unwrap());
    let file = Path::new(&path);
    if !file.exists() {
        downloader
            .lock()
            .unwrap()
            .add_download(url.to_string(), path.clone(), path);
    }
}

// Find a way to return a downloader to user with all download file
async fn download_libraries(
    app: &MinecraftAuth,
    libs: Option<&Vec<Value>>,
    downloader: &RefDownloader,
) {
    let lib_path = format!("{}/libraries/", app.path);

    if let Some(array) = libs {
        array.iter().for_each(|a| {
            if let Some(artifact) = get_artifact(&a) {
                add_download_with_lib_info(artifact, &lib_path, downloader);
            }

            if let Some(classifiers) = get_classifiers(&a) {
                add_download_with_lib_info(classifiers, &lib_path, downloader);
            }
        });
    }
}

async fn download_client(
    app: &MinecraftAuth,
    client: &Value,
    downloader: &RefDownloader,
    version: &str,
) {
    let path = format!("{}/clients/{}/client.jar", app.path, version);
    if !Path::new(&path).exists() {
        downloader.lock().unwrap().add_download(
            client["url"].as_str().unwrap().to_string(),
            path.clone(),
            path,
        );
    }
}

async fn download_assets(app: &MinecraftAuth, assets: &Value, downloader: &RefDownloader) {
    let id = assets["id"].as_str().unwrap();

    let path = format!("{}/assets", app.path);
    if let Ok(_) = download_manifest(&format!("{}/indexes/", path), assets["url"].as_str().unwrap(), &id).await {
        if let Some(manifest) = version_manifest(app, &id) {
            for m in manifest["objects"].as_object().unwrap() {
                let hash = m.1["hash"].as_str().unwrap();
                let f = &hash[..2];
                let p = format!("{}/objects/{}/{}", path, f, hash);
                let url = format!("http://resources.download.minecraft.net/{}/{}", f, hash);

                if !Path::new(&p).exists() {
                    downloader.lock().unwrap().add_download(url, p.clone(), p);
                }
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
                    let path = format!("{}/versions/", app.path);
                    match download_manifest(&path, v["url"].as_str().unwrap(), id).await {
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
                    find_and_install_minecraft_version(app, &version, &manifest, &downloader).await;
                } else {
                    println!("[Error] Can't find manifest_version file");
                }
            }
            Err(err) => println!("[Error] {}", err),
        }
    }
}
