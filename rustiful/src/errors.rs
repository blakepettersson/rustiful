use std::fmt::*;
use std::error::Error;
use status::Status;

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

#[derive(Debug)]
pub enum RequestError<T> where T: Error + Sized + Send {
    NoBody,
    NotFound,
    IdParseError(Box<Error + Send + 'static>),
    RepositoryError(RepositoryError<T>),
    QueryStringParseError(QueryStringParseError)
}

impl <T> RequestError<T> where T: Error + Sized + Send {
    pub fn status(&self) -> Status {
        match self {
            &RequestError::NotFound => Status::NotFound,
            &RequestError::NoBody => Status::BadRequest,
            &RequestError::IdParseError(_) => Status::BadRequest,
            &RequestError::QueryStringParseError(_) => Status::BadRequest,
            &RequestError::RepositoryError(ref err) => err.status
        }
    }
}

impl <T> Display for RequestError<T> where T: Error + Sized + Send {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            RequestError::NoBody => write!(f, "{}", self.description()),
            RequestError::NotFound => write!(f, "{}", self.description()),
            RequestError::IdParseError(ref err) => write!(f, "{}", err.description()),
            RequestError::RepositoryError(ref err) => write!(f, "{}", err.description()),
            RequestError::QueryStringParseError(ref err) => write!(f, "{}", err.description())
        }
    }
}

impl <T> Error for RequestError<T> where T: Error + Sized + Send {
    fn description(&self) -> &str {
        match *self {
            RequestError::NoBody => NO_BODY,
            RequestError::NotFound => NOT_FOUND,
            RequestError::IdParseError(ref err) => err.description(),
            RequestError::RepositoryError(ref err) => err.description(),
            RequestError::QueryStringParseError(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RepositoryError<T: Error + Sized + Send> {
    pub error: T,
    pub status: Status
}

impl <'a, T> RepositoryError<T> where T: 'a + Error + Sized + Send, Status: for <'b> From<&'b T> {
    pub fn new(error: T) -> RepositoryError<T>
    {
        let status:Status = Status::from(&error);

        RepositoryError {
            error: error,
            status: status
        }
    }
}

impl <T> Display for RepositoryError<T> where T: Error + Sized + Send {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Error in repository: {}", self.error.description())
    }
}

impl <T> Error for RepositoryError<T> where T: Error + Sized + Send {
    fn description(&self) -> &str {
        self.error.description()
    }

    fn cause(&self) -> Option<&Error> {
        self.error.cause()
    }
}
