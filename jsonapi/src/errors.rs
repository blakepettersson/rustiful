use std::fmt::*;
use std::error::Error;
use queryspec::QueryStringParseError;

static NOT_FOUND: &'static str = "Not found";

#[derive(Debug, PartialEq, Eq)]
pub enum RequestError {
    NotFound,
}

impl Display for RequestError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            RequestError::NotFound => write!(f, "{}", NOT_FOUND),
        }
    }
}

impl Error for RequestError {
    fn description(&self) -> &str {
        match *self {
            //RequestError::NotFound(ref id) => id,
            RequestError::NotFound => NOT_FOUND,
        }
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

#[derive(Debug)]
pub struct RepositoryError {
    pub error: Box<Error + Send>,
}

impl Display for RepositoryError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Error in repository: {}", self.error.description())
    }
}

impl From<RequestError> for RepositoryError {
    fn from(value: RequestError) -> Self {
        RepositoryError { error: Box::new(value) }
    }
}

impl From<QueryStringParseError> for RepositoryError {
    fn from(value: QueryStringParseError) -> Self {
        RepositoryError { error: Box::new(value) }
    }
}

impl Error for RepositoryError {
    fn description(&self) -> &str {
        self.error.description()
    }

    fn cause(&self) -> Option<&Error> {
        self.error.cause()
    }
}
