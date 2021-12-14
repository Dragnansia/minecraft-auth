use reqwest::Client;
use serde_json::Value;
use tokio::{
    sync::mpsc::{channel, error::TryRecvError, Receiver},
    task::JoinHandle,
};

pub enum UCStatut {
    User(User),
    Error(String),
}

#[derive(Debug)]
pub struct UConnect {
    pub receiver: Receiver<UCStatut>,
    _thread: JoinHandle<()>,
}

impl UConnect {
    pub fn message(&mut self) -> Result<UCStatut, TryRecvError> {
        self.receiver.try_recv()
    }
}

#[derive(Debug, PartialEq, PartialOrd, Default)]
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
}

pub fn connect_to_mojang(username: String, password: String) -> UConnect {
    let (sender, receiver) = channel(1);
    let thread = tokio::spawn(async move {
        let client = Client::new();
        let body = format!("{{\"agent\": {{\"name\": \"Minecraft\",\"version\":1}},\"username\":\"{}\",\"password\":\"{}\"}}", username, password);
        let res = client
            .post("https://authserver.mojang.com/authenticate")
            .body(body)
            .send()
            .await;

        let data: Value = match res {
            Ok(val) => {
                let data = val.text().await.unwrap_or_default();
                serde_json::from_str(&data).unwrap_or_default()
            }
            Err(err) => {
                let _ = sender.send(UCStatut::Error(err.to_string())).await;
                return;
            }
        };

        if let Some(error) = data["errorMessage"].as_str() {
            let _ = sender.send(UCStatut::Error(error.to_string())).await;
        } else {
            let client_token = data["clientToken"].as_str().unwrap().to_string();
            let access_token = data["accessToken"].as_str().unwrap().to_string();
            let uuid = data["selectedProfile"]["id"].as_str().unwrap().to_string();

            let _ = sender
                .send(UCStatut::User(User::new(
                    username,
                    uuid,
                    client_token,
                    access_token,
                )))
                .await;
        }
    });

    UConnect {
        receiver,
        _thread: thread,
    }
}

mod test {
    #[test]
    fn mojang_connect() {}
}
