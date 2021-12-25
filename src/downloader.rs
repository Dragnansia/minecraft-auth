use crate::handle::main_handle;
use futures::StreamExt;
use reqwest::Client;
use std::{
    cmp::min,
    collections::VecDeque,
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;

#[derive(Debug)]
struct DlInfo {
    pub url: String,
    pub path: String,
    pub id: String,
}

#[derive(Debug, Default, Clone)]
pub struct DlStatut {
    current_download: String,
    percentage: u64,
}

pub type RefDownloader = Arc<Mutex<Downloader>>;

#[derive(Debug)]
pub struct Downloader {
    tasks: Arc<Mutex<VecDeque<DlInfo>>>,
    thread: Arc<Mutex<Option<JoinHandle<()>>>>,
    current_state: Arc<Mutex<DlStatut>>,
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(VecDeque::new())),
            thread: Arc::new(Mutex::new(None)),
            current_state: Arc::new(Mutex::new(DlStatut::default())),
        }
    }

    pub fn new_ref() -> RefDownloader {
        Arc::new(Mutex::new(Self {
            tasks: Arc::new(Mutex::new(VecDeque::new())),
            thread: Arc::new(Mutex::new(None)),
            current_state: Arc::new(Mutex::new(DlStatut::default())),
        }))
    }

    pub fn add_download(&mut self, url: String, path: String, id: String) {
        let task_id = id.clone();
        self.tasks
            .lock()
            .unwrap()
            .push_back(DlInfo { url, path, id });

        println!("[Info] Task [{}] is add", task_id);
        self.start_download();
    }

    pub fn start_download(&mut self) {
        if self.thread.lock().unwrap().is_none() {
            let tasks = Arc::clone(&self.tasks);
            let thread = Arc::clone(&self.thread);
            let dl_statut = Arc::clone(&self.current_state);

            let thread_task = Some(main_handle().spawn(async move {
                loop {
                    let t = tasks.lock().unwrap().pop_front();
                    if let Some(dl_info) = t {
                        let remain = tasks.lock().unwrap().len();
                        println!("[Info] Task [{}] is start, in queue {}", dl_info.id, remain);
                        dl_statut.lock().unwrap().current_download = dl_info.path.clone();

                        match download_file(dl_info.url, dl_info.path, Some(&dl_statut)).await {
                            Ok(_) => println!("[Info] Task [{}] is finish", dl_info.id),
                            Err(err) => println!("[Error] Task [{}] -> {}", dl_info.id, err),
                        }
                    } else {
                        break;
                    }
                }

                println!("[Info] Tasks End");
                *thread.lock().unwrap() = None;
            }));

            println!("[Info] Task thread start");
            *self.thread.lock().unwrap() = thread_task;
        }
    }

    pub fn empty(&self) -> bool {
        self.tasks.lock().unwrap().is_empty() && self.thread.lock().unwrap().is_none()
    }

    pub fn statut(&self) -> DlStatut {
        self.current_state.lock().unwrap().clone()
    }

    pub fn wait(&self) {
        loop {
            if self.empty() {
                break;
            }
        }
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
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

pub async fn download_file(
    url: String,
    path: String,
    dl_statut: Option<&Arc<Mutex<DlStatut>>>,
) -> Result<String, String> {
    let client = Client::new();
    match client.get(&url).send().await {
        Ok(response) => {
            path_for_file(just_path(&path));
            let mut file = match File::create(&path) {
                Ok(fc) => fc,
                Err(err) => {
                    return Err(err.to_string());
                }
            };

            let size = response.content_length().unwrap();
            let mut new: u64 = 0;
            let mut stream = response.bytes_stream();

            while let Some(item) = stream.next().await {
                let chunk = item
                    .map_err(|_| "Error while downloading file bytes".to_string())
                    .unwrap();

                file.write(&chunk)
                    .map_err(|_| "Error while writing to file".to_string())
                    .unwrap();

                new = min(new + (chunk.len() as u64), size);
                if let Some(dl_statut) = dl_statut {
                    dl_statut.lock().unwrap().percentage = new * 100 / size;
                }
            }

            Ok(path)
        }
        Err(err) => Err(err.to_string()),
    }
}
