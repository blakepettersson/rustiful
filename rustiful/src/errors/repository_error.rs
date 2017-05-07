use status::Status;
use std::error::Error;
use std::fmt::*;

#[derive(Debug, PartialEq, Eq)]
pub struct RepositoryError<T: Error + Sized + Send> {
    pub error: T,
    pub status: Status,
}

impl<'a, T> RepositoryError<T>
    where T: 'a + Error + Sized + Send,
          Status: for<'b> From<&'b T>
{
    pub fn new(error: T) -> RepositoryError<T> {
        let status: Status = Status::from(&error);

        RepositoryError {
            error: error,
            status: status,
        }
    }
}

impl<T> Display for RepositoryError<T>
    where T: Error + Sized + Send
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Error in repository: {}", self.error.description())
    }
}

impl<T> Error for RepositoryError<T>
    where T: Error + Sized + Send
{
    fn description(&self) -> &str {
        self.error.description()
    }

    fn cause(&self) -> Option<&Error> {
        Some(&self.error)
    }
}
