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
        match *self {
            IoError(ref error) => write!(f, "{}", error),
        }
    }
}

impl Error for DogstatsdError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            IoError(error) => Some(error),
        }
    }
}

impl From<io::Error> for DogstatsdError {
    fn from(e: io::Error) -> Self {
        IoError(e)
    }
}

#[cfg(test)]
mod tests {
    use super::DogstatsdError;
    use std::io;

    #[test]
    fn test_error_display() {
        let err = DogstatsdError::from(io::Error::new(io::ErrorKind::Other, "oh no!"));
        assert_eq!(format!("{}", err), "oh no!".to_owned());
    }
}
