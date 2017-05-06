use std::error::Error;
use std::fmt::*;
use std::str::FromStr;

#[derive(Debug)]
pub struct IdParseError<T: FromStr>(pub T::Err);

impl <T> Error for IdParseError<T> where T : FromStr + Debug, <T as FromStr>::Err: Error {
    fn description(&self) -> &str {
        self.0.description()
    }

    fn cause(&self) -> Option<&Error> {
        self.0.cause()
    }
}

impl <T> Display for IdParseError<T> where T : FromStr + Debug, <T as FromStr>::Err: Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Error parsing id: {}", self.0.description())
    }
}
