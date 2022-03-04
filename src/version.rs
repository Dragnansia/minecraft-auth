use crate::{
    data::{
        asset::{AssetIndex, Assets},
        download::{Artifact, Classifier},
        library::Library,
        package::Package,
        version::{ManifestVersion, Version},
    },
    downloader::{download_file, FileInfo},
    error::{self, Error},
    MinecraftAuth,
};
use log::info;
use serde::Deserialize;
use std::{fs::File, io::BufReader, path::Path};

fn intern_manifest<T>(p: &str) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
{
    let file = File::open(p)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

pub fn manifest<T>(app: &MinecraftAuth, version: &str) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
{
    intern_manifest::<T>(&format!("{}/versions/{}.json", app.path, version))
}

pub fn version_manifest<T>(app: &MinecraftAuth, version: &str) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
{
    intern_manifest::<T>(&format!("{}/assets/indexes/{}.json", app.path, version))
}

async fn download_manifest(path: &str, url: &str, id: &str) -> Result<(), error::Error> {
    download_file(url.to_string(), format!("{}/{}.json", path, id)).await
}

fn add_download_with_lib_info(
    infos: &Artifact,
    lib_path: &str,
    files: &mut Vec<FileInfo>,
) -> Option<()> {
    let url = infos.url.clone();
    let path = format!("{}{}", lib_path, infos.path.clone().unwrap_or_default());

    let p = Path::new(&path);
    if !p.exists() {
        files.push(FileInfo::new(url.to_string(), path, infos.size));
        return Some(());
    }

    let file = File::open(&path).ok()?;
    if file.metadata().ok()?.len() != infos.size {
        files.push(FileInfo::new(url.to_string(), path, infos.size));
    }

    Some(())
}

// Find a way to return a downloader to user with all download file
async fn download_libraries(
    app: &MinecraftAuth,
    libs: &Vec<Library>,
    files: &mut Vec<FileInfo>,
) -> Option<()> {
    let lib_path = format!("{}/libraries/", app.path);

    libs.iter().for_each(|lib| {
        if let Some(artifact) = &lib.downloads.artifact {
            add_download_with_lib_info(artifact, &lib_path, files);
        }

        if let Some(Classifier::Simple(classifiers)) = &lib.downloads.classifiers {
            add_download_with_lib_info(classifiers, &lib_path, files);
        }
    });

    Some(())
}

async fn download_client(
    app: &MinecraftAuth,
    client: &Artifact,
    version: &str,
    files: &mut Vec<FileInfo>,
) -> Option<()> {
    let path = format!("{}/clients/{}/client.jar", app.path, version);
    let file = File::open(&path).ok()?;
    let p = Path::new(&path);

    if !p.exists() || file.metadata().ok()?.len() != client.size {
        files.push(FileInfo::new(client.url.clone(), path, client.size));
    }

    Some(())
}

async fn download_assets(
    app: &MinecraftAuth,
    assets: &AssetIndex,
    files: &mut Vec<FileInfo>,
) -> Result<(), Error> {
    let id = &assets.id;
    let url = &assets.url;
    let path = format!("{}/assets", app.path);
    let indexes_path = format!("{}/indexes/", path);

    download_manifest(&indexes_path, url, id).await?;

    let assets: Assets = version_manifest(app, id)?;
    files.append(
        &mut assets
            .objects
            .iter()
            .filter_map(|o| {
                let hash = o.1.hash.clone();
                let b = &hash[..2];
                let p = format!("{}/objects/{}/{}", path, b, hash);
                let url = format!("http://resources.download.minecraft.net/{}/{}", b, hash);
                let size = o.1.size;

                let path = Path::new(&p);
                let file = File::open(&p).ok()?;

                if path.exists() || file.metadata().ok()?.len() != size {
                    return None;
                }

                Some(FileInfo::new(url, p, size))
            })
            .collect(),
    );

    Ok(())
}

async fn find_and_install_minecraft_version(
    app: &MinecraftAuth,
    version: &str,
    versions: &Vec<Version>,
    files: &mut Vec<FileInfo>,
) -> Result<(), error::Error> {
    let v = versions
        .iter()
        .find(|v| version == v.id)
        .ok_or("No version found")?;

    let package: Package = match manifest(app, &v.id) {
        Ok(m) => m,
        _ => {
            let path = format!("{}/versions/", app.path);
            download_manifest(&path, &v.url, &v.id).await?;

            manifest(app, &v.id)?
        }
    };

    download_libraries(app, &package.libraries, files).await;
    download_client(app, &package.downloads.client, version, files).await;
    download_assets(app, &package.asset_index, files).await?;

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

    let manifest: ManifestVersion = match manifest(app, "manifest_version") {
        Ok(manifest) => manifest,
        _ => {
            let path = format!("{}/versions", app.path);
            download_manifest(
                &path,
                "https://launchermeta.mojang.com/mc/game/version_manifest.json",
                "manifest_version",
            )
            .await?;

            manifest(app, "manifest_version")?
        }
    };

    find_and_install_minecraft_version(app, &version, &manifest.versions, &mut files).await?;

    Ok(files)
}
