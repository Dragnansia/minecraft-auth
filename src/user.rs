use reqwest::Client;
use serde_json::Value;
use tokio::{
    sync::mpsc::{channel, error::TryRecvError, Receiver, Sender},
    task::JoinHandle,
};

pub enum UCStatut {
    User(User),
    RequestError(String),
    ConnectionError(String),
}

#[derive(Debug)]
pub struct UConnect<R, T> {
    pub receiver: Receiver<R>,
    pub _thread: JoinHandle<T>,
}

impl<R, T> UConnect<R, T> {
    pub fn message(&mut self) -> Result<R, TryRecvError> {
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

async fn intern_connect(username: String, password: String, sender: Sender<UCStatut>) {
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
            let _ = sender.send(UCStatut::RequestError(err.to_string())).await;
            return;
        }
    };

    if let Some(error) = data["errorMessage"].as_str() {
        let _ = sender
            .send(UCStatut::ConnectionError(error.to_string()))
            .await;
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
}

pub fn connect_to_mojang(username: String, password: String) -> UConnect<UCStatut, ()> {
    let (sender, receiver) = channel(1);
    let thread = tokio::spawn(async move { intern_connect(username, password, sender).await });

    UConnect {
        receiver,
        _thread: thread,
    }
}

/*
 * Work but it's not start on Tokio Runtime environment
 * and if Runtime::new() is used, the sender and the thread is closed/drop
 */
#[test]
mod test {
    use super::{connect_to_mojang, UCStatut, User};
    use tokio::sync::mpsc::error::TryRecvError;

    #[test]
    fn mojang_connect() {
        let mut uconnect = connect_to_mojang("".to_owned(), "".to_owned());

        loop {
            match uconnect.message() {
                Ok(statut) => {
                    match statut {
                        UCStatut::User(user) => assert_ne!(user, User::default()),
                        UCStatut::Error(err) => assert!(false, "[ERROR] {}", err),
                    };
                    break;
                }
                Err(err) => {
                    assert_ne!(err, TryRecvError::Disconnected);
                }
            }
        }
    }
}
