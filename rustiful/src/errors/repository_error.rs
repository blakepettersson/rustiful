use std::error::Error;
use std::fmt::*;

#[derive(Debug, PartialEq, Eq)]
/// This is a wrapper for user error types for `JsonGet::Error`, `JsonPost::Error` etc.
///
/// This is used to wrap a user supplied error type and gets converted to a `JsonApiError` later on.
pub struct RepositoryError<T: Error + Sized + Send, Status> {
    pub error: T,
    pub status: Status,
}

impl<'a, T, Status> RepositoryError<T, Status>
    where T: 'a + Error + Sized + Send,
          Status: for<'b> From<&'b T>
{
    pub fn new(error: T) -> RepositoryError<T, Status> {
        let status: Status = Status::from(&error);

        RepositoryError {
            error: error,
            status: status,
        }
    }
}

impl<T, Status> Display for RepositoryError<T, Status>
    where T: Error + Sized + Send
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Error in repository: {}", self.error.description())
    }
}

impl<T, Status> Error for RepositoryError<T, Status>
    where T: Error + Sized + Send, Status: Debug
{
    fn description(&self) -> &str {
        self.error.description()
    }

    fn cause(&self) -> Option<&Error> {
        Some(&self.error)
    }
}
