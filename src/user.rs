use reqwest;

#[derive(Debug)]
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

impl Default for User {
    fn default() -> Self {
        Self {
            username: "".to_owned(),
            uuid: "".to_owned(),
            client_token: "".to_owned(),
            access_token: "".to_owned(),
        }
    }
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

/// Callback for connection with error or the User data
type ConnectCallBack = fn(Result<User, &'static str>);

// Send a post request to connect
// get response and send value on callback or return
// need to be a async function or used multi threading with
// callback
///
///
/// # Example
/// ```
/// fn cb(Result<User, &'static str>) {}
///
/// fn functon() {
///     try_connect("Username".to_string(), "Password".to_string(), cb);
///     // or
///     try_connect("Username".to_string(), "Password".to_string(), |res| {});
/// }
/// ```
pub fn try_connect(username: String, password: String, callback: ConnectCallBack) {
    if username.is_empty() || password.is_empty() {
        callback(Err("Username or password is empty"));
    }

    // Move this on other file
    let reqwest_client = reqwest::Client::new();
    let res = reqwest_client
        .post("https://authserver.mojang.com/authenticate")
        .body(format!(
            "{{\"agent\":{{\"name\":\"Minecraft\",\"version\":1}},\"username\": {},\"password\": {} }}",
            username, password
        )).send();

    callback(Ok(User::default()))
}

#[cfg(test)]
mod test {
    use super::try_connect;

    #[test]
    fn connect() {
        try_connect("".to_string(), "".to_string(), |res| {
            println!("{:?}", res.unwrap());
        });
    }
}
