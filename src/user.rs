#[derive(Debug)]
pub struct User {
    pub username: String,
    pub uuid: String,
    pub client_token: String,
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

type ConnectCallBack = fn(Result<User, &'static str>);

// Send a post request to connect
// get response and send value on callback or return
// need to be a async function or used multi threading with
// callback
pub fn try_connect(username: String, password: String, callback: ConnectCallBack) {
    if username.is_empty() || password.is_empty() {
        callback(Err("Username or password is empty"));
    }

    callback(Ok(User::default()))
}

#[cfg(test)]
mod test {
    use super::{try_connect, User};

    #[test]
    fn connect() {
        try_connect("".to_string(), "".to_string(), result);
    }

    fn result(res: Result<User, &'static str>) {
        println!("{:?}", res.unwrap());
    }
}
