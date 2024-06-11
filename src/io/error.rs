use std::fmt::Display;

use http::header::{InvalidHeaderName, InvalidHeaderValue};
use tokio::io;

use crate::gateway;

pub fn error<E: Into<CustomError>>(data: E) -> io::Error {
    io::Error::new(io::ErrorKind::Other, data.into())
}

#[derive(Debug)]
pub enum CustomError {
    RequestStatusLine(RequestStatusLine),
    ResponseStatusLine(ResponseStatusLine),
    Headers(Headers),
    PeerConnection,
    MutexPoison,
    App(gateway::Error),
}

impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for CustomError {}

unsafe impl Send for CustomError {}
unsafe impl Sync for CustomError {}

#[derive(Debug)]

pub enum RequestStatusLine {
    MissingStatusLine,
    MissingMethod,
    MissingPath,
    MissingVersion,
    InvalidMethod,
}

impl From<RequestStatusLine> for CustomError {
    fn from(value: RequestStatusLine) -> Self {
        CustomError::RequestStatusLine(value)
    }
}

#[derive(Debug)]

pub enum ResponseStatusLine {
    MissingStatusLine,
    MissingStatus,
    MissingVersion,
    InvalidStatus,
}

impl From<ResponseStatusLine> for CustomError {
    fn from(value: ResponseStatusLine) -> Self {
        CustomError::ResponseStatusLine(value)
    }
}

#[derive(Debug)]

pub enum Headers {
    InvalidName(InvalidHeaderName),
    InvalidValue(InvalidHeaderValue),
}

impl From<Headers> for CustomError {
    fn from(value: Headers) -> Self {
        CustomError::Headers(value)
    }
}
