use std::fmt::*;
use std::error::Error;

static NO_BODY: &'static str = "No body";
static NOT_FOUND: &'static str = "Not found";

#[derive(Debug, PartialEq, Eq)]
pub enum QueryStringParseError {
    InvalidParam(String),
    InvalidKeyParam(String),
    InvalidValue(String),
    ParseError(String),
    DuplicateSortKey(String),
    UnImplementedError,
}

static UNIMPLEMENTED: &'static str = "Unimplemented";

impl Display for QueryStringParseError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Query string parse error: {}", description(self))
    }
}

impl Error for QueryStringParseError {
    fn description(&self) -> &str {
        description(self)
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

fn description<'a>(error: &'a QueryStringParseError) -> &'a str {
    use self::QueryStringParseError::*;

    match *error {
        InvalidParam(ref desc) => desc,
        InvalidKeyParam(ref desc) => desc,
        InvalidValue(ref desc) => desc,
        ParseError(ref desc) => desc,
        DuplicateSortKey(ref desc) => desc,
        UnImplementedError => UNIMPLEMENTED,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RequestError {
    NoBody,
    NotFound
}

impl Display for RequestError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            RequestError::NoBody => write!(f, "{}", self.description()),
            RequestError::NotFound => write!(f, "{}", self.description()),
        }
    }
}

impl Error for RequestError {
    fn description(&self) -> &str {
        match *self {
            RequestError::NoBody => NO_BODY,
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
