use ez_io::error::{MagicNumberCheckError, WrongMagicNumber};
use std::error::Error;
use std::fmt;
use std::io::Error as IOError;

#[derive(Debug)]
pub struct UnknownSubSection {
    pub in_section: u32,
    pub failed_to_match: u32,
}

impl Error for UnknownSubSection {
    fn description(&self) -> &str {
        "Some sub-section magic number did not match to anything known/handled by a section."
    }
}

impl fmt::Display for UnknownSubSection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Unknown Magic Number: 0x{:X}, In Section Magic Number: 0x{:X}",
            self.failed_to_match, self.in_section
        )
    }
}

#[derive(Debug)]
pub enum ISM2ImportError {
    IO(IOError),
    MagicNumber(WrongMagicNumber),
    UnknownSubSection(UnknownSubSection),
    NoAttributes,
    UnrecognizedBufferType,
}

impl Error for ISM2ImportError {
    fn description(&self) -> &str {
        match *self {
            ISM2ImportError::IO(ref e) => e.description(),
            ISM2ImportError::MagicNumber(ref e) => e.description(),
            ISM2ImportError::UnknownSubSection(ref e) => e.description(),
            ISM2ImportError::NoAttributes => "No Attribute was specified for a Vertex Buffer",
            ISM2ImportError::UnrecognizedBufferType => {
                "Impossible to infer what type of buffer to read in Joint Extra"
            }
        }
    }
}

impl fmt::Display for ISM2ImportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ISM2ImportError::IO(ref e) => e.fmt(f),
            ISM2ImportError::MagicNumber(ref e) => e.fmt(f),
            ISM2ImportError::UnknownSubSection(ref e) => e.fmt(f),
            ISM2ImportError::NoAttributes => write!(f, "No Attributes in Vertices Buffer"),
            ISM2ImportError::UnrecognizedBufferType => {
                write!(f, "Impossible to infer type of buffer")
            }
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
        match e {
            MagicNumberCheckError::IoError(ioe) => ISM2ImportError::IO(ioe),
            MagicNumberCheckError::MagicNumber(mne) => ISM2ImportError::MagicNumber(mne),
        }
    }
}
