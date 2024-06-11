pub(crate) mod entrypoint;
pub mod middleware;
pub mod next;

use std::fmt::Display;

pub use next::Next;
use tokio::io;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new<S: AsRef<str>>(message: S) -> Self {
        Self {
            message: message.as_ref().to_string(),
        }
    }

    pub fn io(error: io::Error) -> Self {
        Self {
            message: format!("Io({error})"),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for Error {}
