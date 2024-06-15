pub(crate) mod entrypoint;
pub mod middleware;
pub mod next;
pub mod origin;
pub mod router;

pub use next::Next;
use std::{fmt::Display, net::TcpStream};
use tokio::{io, net::tcp::OwnedReadHalf};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(String);

type LeftStream = (TcpStream, OwnedReadHalf);

impl Error {
    pub fn new<S: AsRef<str>>(message: S) -> Self {
        Self(message.as_ref().to_string())
    }

    pub fn io(error: io::Error) -> Self {
        Self(format!("Io({error})"))
    }

    pub fn from(value: impl Display) -> Self {
        Self(value.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Error {}
