use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct NotT3z0sStreamError;

impl fmt::Display for NotT3z0sStreamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "not a t3z0s stream")
    }
}

impl error::Error for NotT3z0sStreamError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}
