use std::{fs::read_to_string, path::Path};

use crate::{
    downloader::{download_file, Downloader},
    MinecraftAuth,
};
use serde_json::Value;
use tokio::join;

fn manifest_version(app: &MinecraftAuth, version: &str) -> Option<Value> {
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

async fn download_manifest(app: &MinecraftAuth, url: &str, id: &str) -> Result<String, String> {
    download_file(
        url.to_string(),
        format!("{}/versions/{}.json", app.path, id),
        None,
    )
    .await
}

// async fn intern_download_version(app: &MinecraftAuth, _: Vec<(&str, &str)>) {}

// Find a way to return a downloader to user with all download file
async fn download_libraries(app: &MinecraftAuth, libs: Option<&Vec<Value>>) {
    let lib_path = format!("{}/libraries/", app.path);

    if let Some(array) = libs {
        let mut downloader = Downloader::new();

        // Update this to download just os specific file
        array.iter().for_each(|a| {
            let artifact = &a["downloads"]["artifact"];
            let url = artifact["url"].as_str().unwrap().to_string();
            let path = format!(
                "{}/libraries/{}",
                app.path,
                artifact["path"].as_str().unwrap()
            );
            downloader.add_download(url, path.clone(), path);
        });

        // Don't loop here
        loop {
            if downloader.empty() {
                break;
            }
        }
    }
}

async fn download_client(app: &MinecraftAuth, client: &Value) {}

async fn download_assets(app: &MinecraftAuth, assets: &Value) {}

async fn find_and_install_minecraft_version(app: &MinecraftAuth, version: &str, manifest: &Value) {
    if let Some(versions) = manifest["versions"].as_array() {
        for v in versions {
            let id = v["id"].as_str().unwrap();
            if version == id {
                if let Some(m) = manifest_version(app, id) {
                    let lib = download_libraries(app, m["libraries"].as_array());

                    join!(lib);
                } else {
                    let v_manifest = v["url"].as_str().unwrap();
                    match download_manifest(app, v_manifest, id).await {
                        Ok(_) => {}
                        Err(err) => println!("[Error] {}", err),
                    }
                }
            }
        }
    } else {
        println!("Can't find versions on manifest json");
    }
}

pub async fn download_version(app: &MinecraftAuth, version: String) {
    if let Some(manifest) = manifest_version(app, "manifest_version") {
        find_and_install_minecraft_version(app, &version, &manifest).await;
    } else {
        match download_manifest(
            app,
            "https://launchermeta.mojang.com/mc/game/version_manifest.json
            ",
            "manifest_version",
        )
        .await
        {
            Ok(_) => {
                if let Some(manifest) = manifest_version(app, "manifest_version") {
                    find_and_install_minecraft_version(app, &version, &manifest).await;
                } else {
                    println!("[Error] Can't find manifest_version file");
                }
            }
            Err(err) => println!("[Error] {}", err),
        }
    }
}

#[cfg(test)]
mod test {
    use super::download_version;
    use crate::{
        downloader::{self, Downloader},
        MinecraftAuth,
    };
    use futures::executor::block_on;

    #[test]
    fn dl_version() {
        if let Some(app) = MinecraftAuth::new_just_name("Launcher".to_string()) {
            let mut manifest = Downloader::new();
            manifest.add_download(
                "https://launchermeta.mojang.com/mc/game/version_manifest.json".to_owned(),
                format!("{}/versions/manifest_version.json", app.path),
                "manifest_download".to_owned(),
            );
            let dl = download_version(&app, "".to_string());

            block_on(dl);
        }
    }
}
