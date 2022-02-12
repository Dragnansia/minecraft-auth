use crate::{error, MinecraftAuth};
use reqwest::Client;
use serde_json::{Map, Value};
use std::{
    fs::{read_to_string, File},
    io,
    io::{Read, Write},
    path::Path,
};
use tokio::{
    sync::mpsc::{channel, error::TryRecvError, Receiver, Sender},
    task::JoinHandle,
};

/// This is enum for connection status
/// when you try to connect to a account
#[derive(Debug)]
pub enum UCStatus {
    /// return the user connection information
    User(User),

    /// Error of reqwest when tried to send request
    /// to minecraft api
    RequestError(String),

    /// Error of minecraft api when try to connect
    ConnectionError(String),

    /// Other error like channel close for receiver
    OtherError(String),

    /// Just connection don't have error and is just not finish
    /// Is Here if you want to do something when is not ready
    Waiting,
}

/// This is struct to save receiver and thread
/// where connection is currently working
#[derive(Debug)]
pub struct UConnect {
    receiver: Receiver<UCStatus>,
    _thread: JoinHandle<()>,
}

impl UConnect {
    /// Send UCStatus of the latest result of receiver
    ///
    /// # Example
    /// ```
    /// let u_connect = connect_to_mojang("Username".to_owned(), "Password".to_owned());
    ///
    /// loop {
    ///     match u_connect.message() {
    ///         UCStatus::User(u) => println!("{:?}", u),
    ///         UCStatus::RequestError(err) => println!("{}", err),
    ///         UCStatus::ConnectionError(err) => println!("{}", err),
    ///         UCStatus::OtherError(err) => {},
    ///         UCStatus::Waiting => {},
    ///     }
    /// }
    /// ```
    pub fn message(&mut self) -> UCStatus {
        match self.receiver.try_recv() {
            Ok(r) => r,
            Err(err) => {
                if err == TryRecvError::Disconnected {
                    UCStatus::OtherError(err.to_string())
                } else {
                    UCStatus::Waiting
                }
            }
        }
    }
}

/// Minecraft user information for playing game
#[derive(Debug, PartialEq, PartialOrd, Default, Clone)]
pub struct User {
    /// Email or Pseudo
    pub username: String,

    /// UUID of the current player
    pub uuid: String,

    /// Client token get on connection
    pub client_token: String,

    /// Access token for this account
    pub access_token: String,
}

impl User {
    pub fn new(username: String, uuid: String, client_token: String, access_token: String) -> Self {
        Self {
            username,
            uuid,
            client_token,
            access_token,
        }
    }

    pub fn from_config(app: &MinecraftAuth, username: String) -> Option<Self> {
        let p = format!("{}/users_accounts.json", app.path);
        let path = Path::new(&p);
        if !path.exists() || !path.is_file() {
            return None;
        }

        let file_content = read_to_string(path).ok()?;
        let root: Value = serde_json::from_str(&file_content).ok()?;
        let user = root["users"].get(&username)?;

        Some(Self {
            username,
            uuid: user["uuid"].as_str()?.to_string(),
            client_token: user["client_token"].as_str()?.to_string(),
            access_token: user["access_token"].as_str()?.to_string(),
        })
    }

    pub fn last_from_config(app: &MinecraftAuth) -> Option<Self> {
        let p = format!("{}/users_accounts.json", app.path);
        let path = Path::new(&p);
        if !path.exists() || !path.is_file() {
            return None;
        }

        let file_content = read_to_string(path).ok()?;
        let root = serde_json::from_str::<Value>(&file_content).ok()?;
        let user = root["users"].as_object()?.iter().last()?;

        Some(Self {
            username: user.0.clone(),
            uuid: user.1["uuid"].as_str()?.to_string(),
            client_token: user.1["client_token"].as_str()?.to_string(),
            access_token: user.1["access_token"].as_str()?.to_string(),
        })
    }

    pub fn save_on_file(&self, app: &MinecraftAuth) -> Result<(), error::Error> {
        let mut file = User::open_user_file(app)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        if content.is_empty() {
            content += "{}";
        }

        let root: Value = serde_json::from_str(&content)?;
        let el = match root {
            Value::Object(mut r) => {
                if let Some(users) = r["users"].as_object_mut() {
                    if let Some(user) = users[&self.username].as_object_mut() {
                        user["uuid"] = Value::String(self.uuid.clone());
                        user["access_token"] = Value::String(self.access_token.clone());
                        user["client_token"] = Value::String(self.client_token.clone());
                    } else {
                        let mut user = Map::new();
                        user.insert(self.username.clone(), Value::Object(self.convert_to_map()));

                        users.insert(self.username.clone(), Value::Object(user));
                    }
                } else {
                    let mut user = Map::new();
                    user.insert(self.username.clone(), Value::Object(self.convert_to_map()));
                    r.insert("users".into(), Value::Object(user));
                }

                Value::Object(r)
            }
            v => v,
        };

        let new_content = serde_json::to_string(&el)?;
        file.write_all(new_content.as_bytes())?;

        Ok(())
    }

    fn convert_to_map(&self) -> Map<String, Value> {
        let mut user_info = Map::new();
        user_info.insert("uuid".to_string(), Value::String(self.uuid.clone()));
        user_info.insert(
            "access_token".to_string(),
            Value::String(self.access_token.clone()),
        );
        user_info.insert(
            "client_token".to_string(),
            Value::String(self.client_token.clone()),
        );

        user_info
    }

    pub fn disconnect(&self, app: &MinecraftAuth) -> Result<(), error::Error> {
        let mut file = User::open_user_file(app)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        if content.is_empty() || content == "\n" {
            return Ok(());
        }

        let root: Value = serde_json::from_str(&content)?;
        let el = match root {
            Value::Object(mut r) => {
                let arr = r["users"].as_object_mut().ok_or("No found 'user' key")?;
                arr.remove(&self.username).ok_or("Can't remove user")?;
                Value::Object(r)
            }
            v => v,
        };

        let new_content = serde_json::to_string(&el).unwrap();
        file.write_all(new_content.as_bytes()).unwrap();

        Ok(())
    }

    fn open_user_file(app: &MinecraftAuth) -> io::Result<File> {
        File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(format!("{}/users_accounts.json", app.path))
    }
}

/// This is the intern connection function for mojang minecraft api
async fn intern_connect(
    username: String,
    password: String,
    sender: Sender<UCStatus>,
) -> Result<(), error::Error> {
    let client = Client::new();
    let body = format!("{{\"agent\": {{\"name\": \"Minecraft\",\"version\":1}},\"username\":\"{}\",\"password\":\"{}\"}}", username, password);
    let res = client
        .post("https://authserver.mojang.com/authenticate")
        .body(body)
        .send()
        .await;

    let data: Value = match res {
        Ok(val) => {
            let data = val.text().await?;
            serde_json::from_str(&data)?
        }
        Err(err) => {
            sender.send(UCStatus::RequestError(err.to_string())).await?;
            return Ok(());
        }
    };

    if let Some(error) = data["errorMessage"].as_str() {
        sender
            .send(UCStatus::ConnectionError(error.to_string()))
            .await?;
    } else {
        let client_token = data["clientToken"]
            .as_str()
            .ok_or("clientToken str")?
            .to_string();
        let access_token = data["accessToken"]
            .as_str()
            .ok_or("accessToken str")?
            .to_string();
        let uuid = data["selectedProfile"]["id"]
            .as_str()
            .ok_or("selectedProfile id str")?
            .to_string();

        sender
            .send(UCStatus::User(User::new(
                username,
                uuid,
                client_token,
                access_token,
            )))
            .await?;
    }

    Ok(())
}

/// Try to connect to mojang api with Username and Password
// Remove this to just use microsoft connect method
pub fn connect_to_mojang(username: String, password: String) -> UConnect {
    let (sender, receiver) = channel(1);
    let thread =
        tokio::spawn(async move { intern_connect(username, password, sender).await.unwrap() });

    UConnect {
        receiver,
        _thread: thread,
    }
}
