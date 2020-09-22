extern crate x11rb;

use std::error::Error as StdError;

#[derive(Debug)]
pub enum Error {
    ConnectionError(x11rb::errors::ConnectionError),
    X11Error(x11rb::protocol::Error),
    Other(&'static str),
}

impl From<x11rb::errors::ReplyError> for Error {
    fn from(value: x11rb::errors::ReplyError) -> Error {
        match value {
            x11rb::rust_connection::ReplyError::X11Error(err) => Error::X11Error(err),
            x11rb::rust_connection::ReplyError::ConnectionError(err) => Error::ConnectionError(err),
        }
    }
}

impl From<x11rb::errors::ConnectionError> for Error {
    fn from(value: x11rb::errors::ConnectionError) -> Error {
        Error::ConnectionError(value)
    }
}

impl From<x11rb::protocol::Error> for Error {
    fn from(value: x11rb::protocol::Error) -> Error {
        Error::X11Error(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConnectionError(err) => std::fmt::Display::fmt(&err, f),
            Error::X11Error(err) => std::fmt::Debug::fmt(&err, f),
            Error::Other(msg) => write!(f, "({})", msg),
        }
    }
}

impl StdError for Error {}

pub fn log_if_failed<_T, ERR: StdError + ?Sized>(res: &Result<_T, Box<ERR>>) {
    if let Err(error) = res {
        eprintln!("Error: {:?}", error);
    }
}
