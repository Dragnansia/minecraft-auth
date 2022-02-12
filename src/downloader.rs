use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File},
    io::Write,
    iter::Sum,
    path::Path,
};

use crate::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub url: String,
    pub path: String,
    pub size: u64,
}

impl<'a> Sum<&'a FileInfo> for u64 {
    fn sum<I: Iterator<Item = &'a FileInfo>>(iter: I) -> Self {
        iter.map(|v| v.size).collect::<Vec<u64>>().iter().sum()
    }
}

impl FileInfo {
    pub fn new(url: String, path: String, size: u64) -> Self {
        Self { url, path, size }
    }
}

fn just_path(path: &str) -> &str {
    let filename_size = path.split('/').last().unwrap().len();
    &path[..path.len() - filename_size]
}

fn path_for_file(path: &str) {
    let f = Path::new(path);
    if !f.exists() {
        create_folder(path);
    }
}

fn create_folder(folder: &str) {
    create_dir_all(folder).unwrap();
}

pub async fn download_file(url: String, path: String) -> Result<(), error::Error> {
    let client = Client::new();
    let response = client.get(&url).send().await?;

    path_for_file(just_path(&path));
    let mut file = File::create(&path)?;

    let mut stream = response.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item
            .map_err(|_| "Error while downloading file bytes".to_string())
            .unwrap();

        file.write(&chunk)
            .map_err(|_| "Error while writing to file".to_string())
            .unwrap();
    }

    Ok(())
}
