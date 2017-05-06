use errors::query_string_parse_error::QueryStringParseError;
use errors::repository_error::RepositoryError;
use status::Status;
use std::error::Error;
use std::fmt::*;

static NO_BODY: &'static str = "No body";
static NOT_FOUND: &'static str = "Not found";

#[derive(Debug)]
pub enum RequestError<T>
    where T: Error + Sized + Send
{
    NoBody,
    NotFound,
    IdParseError(Box<Error + Send + 'static>),
    RepositoryError(RepositoryError<T>),
    QueryStringParseError(QueryStringParseError),
}

impl<T> RequestError<T>
    where T: Error + Sized + Send
{
    pub fn status(&self) -> Status {
        match *self {
            RequestError::NotFound => Status::NotFound,
            RequestError::RepositoryError(ref err) => err.status,
            _ => Status::BadRequest,
        }
    }
}

impl<T> Display for RequestError<T>
    where T: Error + Sized + Send
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            RequestError::NoBody => write!(f, "{}", self.description()),
            RequestError::NotFound => write!(f, "{}", self.description()),
            RequestError::IdParseError(ref err) => write!(f, "{}", err.description()),
            RequestError::RepositoryError(ref err) => write!(f, "{}", err.description()),
            RequestError::QueryStringParseError(ref err) => write!(f, "{}", err.description()),
        }
    }
}

impl<T> Error for RequestError<T>
    where T: Error + Sized + Send
{
    fn description(&self) -> &str {
        match *self {
            RequestError::NoBody => NO_BODY,
            RequestError::NotFound => NOT_FOUND,
            RequestError::IdParseError(ref err) => err.description(),
            RequestError::RepositoryError(ref err) => err.description(),
            RequestError::QueryStringParseError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
