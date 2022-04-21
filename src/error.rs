use crate::{instance::InstanceCreateError, user::UCStatus};
use std::io;
use tokio::sync::mpsc::error::SendError;
use zip::result::ZipError;

#[derive(Debug)]
pub enum Error {
    File(io::Error),
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error),
    SendError(SendError<UCStatus>),
    InstanceCreate(InstanceCreateError),
    Zip(ZipError),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Error::File(f) => f.to_string(),
            Error::Reqwest(r) => r.to_string(),
            Error::SerdeJson(s) => s.to_string(),
            Error::SendError(s) => s.to_string(),
            Error::InstanceCreate(ic) => ic.to_string(),
            Error::Zip(z) => z.to_string(),
            Error::Other(o) => o.to_string(),
        };

        f.write_str(&message)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::File(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::Reqwest(error)
    }
}

impl From<&str> for Error {
    fn from(error: &str) -> Self {
        Self::Other(error.into())
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Self::Other(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::SerdeJson(error)
    }
}

impl From<SendError<UCStatus>> for Error {
    fn from(error: SendError<UCStatus>) -> Self {
        Self::SendError(error)
    }
}

impl From<InstanceCreateError> for Error {
    fn from(error: InstanceCreateError) -> Self {
        Self::InstanceCreate(error)
    }
}

impl From<ZipError> for Error {
    fn from(error: ZipError) -> Self {
        Self::Zip(error)
    }
}
