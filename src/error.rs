use std::fmt::{Display, Formatter, self};


pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    message:  String
}

impl From<bluer::Error> for Error {
    fn from(e: bluer::Error) -> Error {
        new(e.message)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        new(e.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub(crate) fn new(message: String) -> Error {
    Error {
        message,
    }
}