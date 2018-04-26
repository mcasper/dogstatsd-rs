use std::error::Error;
use std::{fmt, io};

/// This type represents the possible errors that can occur while
/// sending DogstatsD metrics.
#[derive(Debug)]
pub enum DogstatsdError {
    /// Chained IO errors.
    IoError(io::Error),
}

use self::DogstatsdError::*;

impl fmt::Display for DogstatsdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Error for DogstatsdError {
    fn description(&self) -> &str {
        match *self {
            IoError(ref error) => error.description(),
        }
    }
}

impl From<io::Error> for DogstatsdError {
    fn from(e: io::Error) -> Self {
        IoError(e)
    }
}
