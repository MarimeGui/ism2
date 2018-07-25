use magic_number::MagicNumberCheckError;
use std::error::Error;
use std::fmt;
use std::io::Error as IOError;

#[derive(Debug)]
pub enum ISM2ImportError {
    IO(IOError),
    MagicNumber(MagicNumberCheckError),
}

impl Error for ISM2ImportError {
    fn description(&self) -> &str {
        match *self {
            ISM2ImportError::IO(ref e) => e.description(),
            ISM2ImportError::MagicNumber(ref e) => e.description(),
        }
    }
}

impl fmt::Display for ISM2ImportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ISM2ImportError::IO(ref e) => e.fmt(f),
            ISM2ImportError::MagicNumber(ref e) => e.fmt(f),
        }
    }
}

impl From<IOError> for ISM2ImportError {
    fn from(e: IOError) -> ISM2ImportError {
        ISM2ImportError::IO(e)
    }
}

impl From<MagicNumberCheckError> for ISM2ImportError {
    fn from(e: MagicNumberCheckError) -> ISM2ImportError {
        ISM2ImportError::MagicNumber(e)
    }
}
