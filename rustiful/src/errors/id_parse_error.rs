use std::error::Error;
use std::fmt::*;
use std::str::FromStr;

#[derive(Debug)]
/// A new-type wrapper for `FromStr::Err`.
///
/// This is a wrapper for a `FromStr::Err` type. This is used whenever an attempt to convert an id
/// in `&str` format to a native id type fails, and it then gets converted to a `JsonApiError`.
pub struct IdParseError<T: FromStr>(pub T::Err);

impl<T> Error for IdParseError<T>
    where T: FromStr + Debug,
          <T as FromStr>::Err: Error
{
    fn description(&self) -> &str {
        self.0.description()
    }

    fn cause(&self) -> Option<&Error> {
        Some(&self.0)
    }
}

impl<T> Display for IdParseError<T>
    where T: FromStr + Debug,
          <T as FromStr>::Err: Error
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Error parsing id: {}", self.0)
    }
}
