use status::Status;
use std::error::Error;
use std::fmt::*;

static NO_BODY: &'static str = "No body";
static NOT_FOUND: &'static str = "Not found";

#[derive(Debug, Copy, Clone)]
/// Wraps request related errors
///
/// This is a container for HTTP related errors. Currently there's only a variant for not `POST`ing
/// or `PUT`ing a body, or if a resource cannot be found.
pub enum RequestError {
    NoBody,
    NotFound
}

impl RequestError {
    pub fn status(&self) -> Status {
        match *self {
            RequestError::NotFound => Status::NotFound,
            RequestError::NoBody => Status::BadRequest,
        }
    }
}

impl Display for RequestError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            RequestError::NoBody => write!(f, "{}", self.description()),
            RequestError::NotFound => write!(f, "{}", self.description())
        }
    }
}

impl Error for RequestError {
    fn description(&self) -> &str {
        match *self {
            RequestError::NoBody => NO_BODY,
            RequestError::NotFound => NOT_FOUND
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            RequestError::NoBody | RequestError::NotFound => None
        }
    }
}
