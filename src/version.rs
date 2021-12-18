use std::{fs::read_to_string, path::Path};

use crate::{
    downloader::{self, download_file, Downloader},
    MinecraftAuth,
};
use serde_json::Value;

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

pub async fn download_version(app: &MinecraftAuth, version: String) {
    if let Some(manifest) = manifest_version(app, "manifest_version") {
        if let Some(versions) = manifest["versions"].as_array() {
            for v in versions {
                let id = v["id"].as_str().unwrap();
                if version == id {
                    if let Some(_) = manifest_version(app, id) {
                        println!("manifest for {} is found", id);
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
    } else {
        println!("Can't find manifest");
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
