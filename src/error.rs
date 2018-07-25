use ez_io::error::{MagicNumberCheckError, WrongMagicNumber};
use std::error::Error;
use std::fmt;
use std::io::Error as IOError;

#[derive(Debug)]
pub struct UnknownSubSection {
    pub magic_number_section: u32,
    pub magic_number_sub_section: u32,
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
            "Unknown Magic Number: 0x{:X}, Section Magic Number: 0x{:X}",
            self.magic_number_sub_section, self.magic_number_section
        )
    }
}

#[derive(Debug)]
pub enum ISM2ImportError {
    IO(IOError),
    MagicNumber(WrongMagicNumber),
    UnknownSubSection(UnknownSubSection),
}

impl Error for ISM2ImportError {
    fn description(&self) -> &str {
        match *self {
            ISM2ImportError::IO(ref e) => e.description(),
            ISM2ImportError::MagicNumber(ref e) => e.description(),
            ISM2ImportError::UnknownSubSection(ref e) => e.description(),
        }
    }
}

impl fmt::Display for ISM2ImportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ISM2ImportError::IO(ref e) => e.fmt(f),
            ISM2ImportError::MagicNumber(ref e) => e.fmt(f),
            ISM2ImportError::UnknownSubSection(ref e) => e.fmt(f),
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
