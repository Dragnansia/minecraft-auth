use crate::{
    downloader::{download_file, DlStatut, ThreadData, ThreadStatut},
    MinecraftAuth,
};
use serde_json::Value;
use std::{fs::read_to_string, path::Path};
use tokio::sync::mpsc::{channel, Sender};

pub fn update_manifest_version(
    app: &MinecraftAuth,
    force: bool,
) -> Option<ThreadData<DlStatut, ()>> {
    let path_mv = format!("{}/meta/minecraft/version_manifest.json", app.path);
    if !Path::new(&path_mv).exists() || force {
        Some(download_file(
            "https://launchermeta.mojang.com/mc/game/version_manifest.json".to_owned(),
            path_mv,
        ))
    } else {
        None
    }
}

async fn async_loop_dl_statut(td: &mut ThreadData<DlStatut, ()>, sender: &Sender<DlStatut>) {
    loop {
        match td.message() {
            Ok(s) => {
                let _ = sender.send(s).await;
            }
            Err(e) => {
                if e == ThreadStatut::Closed {
                    break;
                }
            }
        }
    }
}

async fn download_client(app: &MinecraftAuth, url: String, sender: &Sender<DlStatut>) {
    // download_file(url, format!("{}/client.jar", app.path));
}

/// Download all file for a specifique version
async fn intern_download_version(
    app: MinecraftAuth,
    url: String,
    version: String,
    sender: Sender<DlStatut>,
) {
    if let Some(mut up) = update_manifest_version(&app, false) {
        async_loop_dl_statut(&mut up, &sender).await;
    }

    let vfile = format!("{}/meta/minecraft/{}/index.json", app.path, version);
    let mut file = download_file(url, vfile.clone());
    async_loop_dl_statut(&mut file, &sender).await;

    match read_to_string(vfile) {
        Ok(file) => {
            match serde_json::from_str::<Value>(&file) {
                Ok(json) => {
                    println!("{:?}", json);
                }
                Err(err) => sender
                    .send(DlStatut::Error(err.to_string()))
                    .await
                    .unwrap_or_default(),
            };
        }
        Err(err) => sender
            .send(DlStatut::Error(err.to_string()))
            .await
            .unwrap_or_default(),
    }
}

/// Download a specifique version of the game
/// Return the pourcentage of download version
fn download_version(app: &MinecraftAuth, url: String, version: String) -> ThreadData<DlStatut, ()> {
    let (tx, rx) = channel(1);
    let appclone = app.clone();

    let _ = async move { intern_download_version(appclone, url, version, tx).await };

    ThreadData {
        id: rand::random::<i128>(),
        receiver: rx,
        _thread: None,
    }
}

#[test]
mod test {
    use super::download_version;
    use crate::{downloader::DlStatut, MinecraftAuth};
    use tokio::sync::mpsc::error::TryRecvError;

    #[test]
    fn dl_version() {
        let app = MinecraftAuth::default();
        let mut dl = download_version(&app, "https://launchermeta.mojang.com/v1/packages/b0bdc637e4c4cbf0501500cbaad5a757b04848ed/1.18.1.json".to_owned(), "1.18.1".to_owned());

        loop {
            match dl.message() {
                Ok(r) => match r {
                    DlStatut::Percentage(p) => println!("Percentage {}", r),
                    DlStatut::Error(err) => {
                        println!("Error: {}", err);
                        break;
                    }
                    DlStatut::Finish => break,
                },
                Err(err) => {
                    if err = TryRecvError::Disconnected {
                        break;
                    }
                }
            }
        }
    }
}
