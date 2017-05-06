extern crate bodyparser;

use self::bodyparser::BodyError;
use std::error::Error;
use std::fmt::*;

#[derive(Debug)]
pub struct BodyParserError(pub BodyError);

impl Error for BodyParserError {
    fn description(&self) -> &str {
        self.0.description()
    }

    fn cause(&self) -> Option<&Error> {
        self.0.cause()
    }
}


impl Display for BodyParserError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Error when parsing json: {}", self.0.description())
    }
}
