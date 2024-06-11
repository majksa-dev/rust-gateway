pub(crate) mod entrypoint;
pub mod middleware;
pub mod next;
pub mod origin;

use std::fmt::Display;

use http::StatusCode;
pub use next::Next;
use tokio::io;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),
    HttpStatus(StatusCode),
}

impl Error {
    pub fn new<S: AsRef<str>>(message: S) -> Self {
        Self::Message(message.as_ref().to_string())
    }

    pub fn io(error: io::Error) -> Self {
        Self::Message(format!("Io({error})"))
    }

    pub fn status(status: StatusCode) -> Self {
        Self::HttpStatus(status)
    }

    pub fn get_status_code(&self) -> Option<&StatusCode> {
        match self {
            Self::HttpStatus(status) => Some(status),
            _ => None,
        }
    }
}

impl From<StatusCode> for Error {
    fn from(value: StatusCode) -> Self {
        Self::status(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(message) => message.fmt(f),
            Self::HttpStatus(status) => {
                "Gateway returned status code ".fmt(f)?;
                status.fmt(f)?;
                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {}
