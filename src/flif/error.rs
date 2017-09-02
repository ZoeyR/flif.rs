use std::result;
use std::error;
use std::fmt;
use std::io;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    InvalidHeader { desc: &'static str },
    UnknownOptionalMetadata([u8; 4]),
    UnknownRequiredMetadata(u8),
    InvalidMetadata(String),
    Unimplemented(&'static str),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::InvalidHeader { desc } => desc,
            &Error::Io(ref err) => err.description(),
            &Error::UnknownOptionalMetadata(_) => "encountered an unknown optional metadata",
            &Error::UnknownRequiredMetadata(_) => "encountered an unknown required metadata",
            &Error::InvalidMetadata(_) => "metadata chunk was not a valid deflate stream",
            &Error::Unimplemented(desc) => desc,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &Error::InvalidHeader { .. } => None,
            &Error::Io(ref err) => Some(err),
            &Error::UnknownOptionalMetadata(_) => None,
            &Error::UnknownRequiredMetadata(_) => None,
            &Error::InvalidMetadata(_) => None,
            &Error::Unimplemented(_) => None,
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
            &Error::UnknownOptionalMetadata(ref header) => write!(
                fmt,
                "unknown metadata header encountered: {}",
                String::from_utf8_lossy(header)
            ),
            &Error::UnknownRequiredMetadata(ref byte) => write!(
                fmt,
                "unknown required metadata section with byte header: {}",
                byte
            ),
            &Error::InvalidMetadata(_) => {
                write!(fmt, "metadata content was not a valid deflate stream")
            }
            &Error::Unimplemented(desc) => write!(fmt, "{}", desc),
        }
    }
}
