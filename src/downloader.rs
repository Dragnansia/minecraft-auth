use futures::StreamExt;
use rand::Rng;
use reqwest::Client;
use std::{
    cmp::min,
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};
use tokio::{
    sync::mpsc::{channel, error::TryRecvError, Receiver, Sender},
    task::JoinHandle,
};

#[derive(Debug)]
pub enum DlStatut {
    Percentage(String, u64),
    Error(String),
    Finish,
}

#[derive(Debug, PartialEq)]
pub enum ThreadStatut {
    Closed,
    Waiting,
}

/// Is used to know the current state of the thread
#[derive(Debug)]
pub struct ThreadData<R, T> {
    /// ID of this ThreadData, generate with rand create
    /// (!) need to find a way to get something like thread id
    /// or other specifique information to thread or receiver
    pub id: i128,

    /// Receiver used to get message from thread
    /// Return the specifique Type
    pub receiver: Receiver<R>,

    /// Thread used by Sender<R>, need to be store if we don't
    /// a drop
    pub _thread: Option<JoinHandle<T>>,
}

impl<R, T> PartialEq for ThreadData<R, T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<R, T> ThreadData<R, T> {
    /// The same if `self.receiver.try_recv()` is used
    /// but with a different Error enum.
    ///
    /// You are sure to be a mutable reference to used
    /// correct function
    pub fn message(&mut self) -> Result<R, ThreadStatut> {
        match self.receiver.try_recv() {
            Ok(response) => Ok(response),
            Err(err) => {
                if err == TryRecvError::Disconnected {
                    Err(ThreadStatut::Closed)
                } else {
                    Err(ThreadStatut::Waiting)
                }
            }
        }
    }
}

fn just_path<'a>(path: &'a str) -> &'a str {
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

async fn intern_download_file(url: String, path: String, tx: Sender<DlStatut>) {
    let client = Client::new();
    match client.get(&url).send().await {
        Ok(response) => {
            path_for_file(just_path(&path));
            let mut file = match File::create(&path) {
                Ok(fc) => fc,
                Err(err) => {
                    tx.send(DlStatut::Error(err.to_string())).await.unwrap();
                    return;
                }
            };
            let size = response.content_length().unwrap();
            let mut percentage: u64 = 0;
            let mut stream = response.bytes_stream();

            while let Some(item) = stream.next().await {
                let chunk = item
                    .map_err(|_| -> String { "Error while downloading file".to_string() })
                    .unwrap();

                file.write(&chunk)
                    .map_err(|_| "Error while writing to file".to_string())
                    .unwrap();

                percentage = min(percentage + (chunk.len() as u64), size);

                let _ = tx
                    .send(DlStatut::Percentage(path.clone(), percentage * 100 / size))
                    .await;
            }

            let _ = tx.send(DlStatut::Finish).await;
        }
        Err(err) => {
            let _ = tx.send(DlStatut::Error(err.to_string())).await;
        }
    };
}

/// Download one file and return a ThreadData
pub fn download_file(url: String, path: String) -> ThreadData<DlStatut, ()> {
    let (tx, receiver) = channel(1024);
    let thread = tokio::spawn(async move { intern_download_file(url, path, tx).await });

    ThreadData {
        id: rand::thread_rng().gen::<i128>(),
        receiver,
        _thread: Some(thread),
    }
}
