use reqwest::Client;
use serde_json::Value;
use tokio::{
    sync::mpsc::{channel, error::TryRecvError, Receiver, Sender},
    task::JoinHandle,
};

/// This is enum for connection statut
/// when you try to connect to a account
pub enum UCStatut {
    /// return the user connection information
    User(User),

    /// Error of reqwest when tried to send request
    /// to minecraft api
    RequestError(String),

    /// Error of minecraft api when try to connect
    ConnectionError(String),

    /// Other error like channel close for receiver
    OtherError(String),

    /// Juste connection don't have error and is just not finish
    /// Is Here if you want to do something when is not ready
    Waiting,
}

/// This is struct to save receiver and thread
/// where connection is currently working
#[derive(Debug)]
pub struct UConnect {
    receiver: Receiver<UCStatut>,
    _thread: JoinHandle<()>,
}

impl UConnect {
    /// Send UCStatut of the latest result of receiver
    ///
    /// # Example
    /// ```
    /// let uconnect = mojang_connect("".to_owned(), "".to_owned());
    ///
    /// loop {
    ///     match uconnect.message() {
    ///         UCStatut::User(u) => println!("{:?}", u),
    ///         UCStatut::RequestError(err) => println!("{}", err),
    ///         UCStatut::ConnectionError(err) => println!("{}", err),
    ///         UCStatut::OtherError(err) => {},
    ///         UCStatut::Waiting => {},
    ///     }
    /// }
    /// ```
    pub fn message(&mut self) -> UCStatut {
        match self.receiver.try_recv() {
            Ok(r) => r,
            Err(err) => {
                if err == TryRecvError::Disconnected {
                    UCStatut::OtherError(err.to_string())
                } else {
                    UCStatut::Waiting
                }
            }
        }
    }
}

/// Minecraft user information for playing game
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

/// This is the intern connection function for mojang minecraft api
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

/// Try to connect to mojang api with Username and Password
pub fn connect_to_mojang(username: String, password: String) -> UConnect {
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
mod test {
    use super::{connect_to_mojang, UCStatut, User};

    #[test]
    fn mojang_connect() {
        let mut uconnect = connect_to_mojang("".to_owned(), "".to_owned());

        loop {
            match uconnect.message() {
                UCStatut::User(user) => assert_ne!(user, User::default()),
                UCStatut::RequestError(_) => todo!(),
                UCStatut::ConnectionError(_) => todo!(),
                UCStatut::OtherError(_) => todo!(),
                UCStatut::Waiting => todo!(),
            }
        }
    }
}
