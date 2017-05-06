use std::error::Error;
use std::fmt::*;

#[derive(Debug)]
pub struct FromRequestError<T: Error + Send>(pub T);

impl <T> Error for FromRequestError<T> where T : Error + Send {
    fn description(&self) -> &str {
        self.0.description()
    }

    fn cause(&self) -> Option<&Error> {
        self.0.cause()
    }
}


impl <T> Display for FromRequestError<T> where T : Error + Send {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Error in FromRequest: {}", self.0.description())
    }
}
