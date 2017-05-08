use errors::IdParseError;
use errors::query_string_parse_error::QueryStringParseError;
use errors::repository_error::RepositoryError;
use status::Status;
use std::error::Error;
use std::fmt::*;
use std::str::FromStr;

static NO_BODY: &'static str = "No body";
static NOT_FOUND: &'static str = "Not found";

#[derive(Debug)]
/// The general error type that wraps all errors that can happen when requesting a JsonApi resource.
pub enum RequestError<T, I>
    where T: Error + Sized + Send,
          I: FromStr + Debug,
          <I as FromStr>::Err: Error
{
    NoBody,
    NotFound,
    IdParseError(IdParseError<I>),
    RepositoryError(RepositoryError<T>),
    QueryStringParseError(QueryStringParseError),
}

impl<T, I> RequestError<T, I>
    where T: Error + Sized + Send,
          I: FromStr + Debug,
          <I as FromStr>::Err: Error
{
    pub fn status(&self) -> Status {
        match *self {
            RequestError::NotFound => Status::NotFound,
            RequestError::RepositoryError(ref err) => err.status,
            _ => Status::BadRequest,
        }
    }
}

impl<T, I> Display for RequestError<T, I>
    where T: Error + Sized + Send,
          I: FromStr + Debug,
          <I as FromStr>::Err: Error
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            RequestError::NoBody => write!(f, "{}", self.description()),
            RequestError::NotFound => write!(f, "{}", self.description()),
            RequestError::IdParseError(ref err) => write!(f, "{}", err),
            RequestError::RepositoryError(ref err) => write!(f, "{}", err),
            RequestError::QueryStringParseError(ref err) => write!(f, "{}", err),
        }
    }
}

impl<T, I> Error for RequestError<T, I>
    where T: Error + Sized + Send,
          I: FromStr + Debug,
          <I as FromStr>::Err: Error
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
        match *self {
            RequestError::NoBody | RequestError::NotFound => None,
            RequestError::IdParseError(ref err) => Some(err),
            RequestError::RepositoryError(ref err) => Some(err),
            RequestError::QueryStringParseError(ref err) => Some(err),
        }
    }
}
