use errors::IdParseError;
use errors::QueryStringParseError;
use errors::RepositoryError;
use status::Status;
use std::error::Error;
use std::fmt::*;
use std::str::FromStr;

static NO_BODY: &'static str = "No body";
static NOT_FOUND: &'static str = "Not found";

#[derive(Debug)]
/// The general error type that wraps all errors that can happen when requesting a JsonApi resource.
pub enum RequestError<E> where E: Error + Sized + Send
{
    NoBody,
    NotFound,
    RepositoryError(RepositoryError<E>),
    QueryStringParseError(QueryStringParseError),
}

impl<E> RequestError<E> where E: Error + Sized + Send {
    pub fn status(&self) -> Status {
        match *self {
            RequestError::NotFound => Status::NotFound,
            RequestError::RepositoryError(ref err) => err.status,
            _ => Status::BadRequest,
        }
    }
}

impl<E> Display for RequestError<E> where E: Error + Sized + Send
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            RequestError::NoBody => write!(f, "{}", self.description()),
            RequestError::NotFound => write!(f, "{}", self.description()),
            RequestError::RepositoryError(ref err) => write!(f, "{}", err),
            RequestError::QueryStringParseError(ref err) => write!(f, "{}", err),
        }
    }
}

impl<E> Error for RequestError<E> where E: Error + Sized + Send  {
    fn description(&self) -> &str {
        match *self {
            RequestError::NoBody => NO_BODY,
            RequestError::NotFound => NOT_FOUND,
            RequestError::RepositoryError(ref err) => err.description(),
            RequestError::QueryStringParseError(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            RequestError::NoBody | RequestError::NotFound => None,
            RequestError::RepositoryError(ref err) => Some(err),
            RequestError::QueryStringParseError(ref err) => Some(err),
        }
    }
}
