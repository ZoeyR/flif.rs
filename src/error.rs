use std::result;
use std::error;
use std::fmt;
use std::io;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    InvalidHeader { desc: &'static str },
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::InvalidHeader { desc } => desc,
            &Error::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &Error::InvalidHeader { .. } => None,
            &Error::Io(ref err) => Some(err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::InvalidHeader { desc } => write!(fmt, "FLIF header was invalid: {}", desc),
            &Error::Io(ref err) => write!(fmt, "error reading from stream: {}", err),
        }
    }
}
