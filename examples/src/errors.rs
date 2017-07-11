use diesel;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug)]
/// This error wraps any error returned from the database, along with any errors returned from
/// attempting to convert a `Todo::Resource` (aka the JSONAPI representation of a `Todo`) to a
/// `Todo`. Any error used in `JsonGet`, `JsonIndex` et cetera has to implement `std::error::Error`.
pub enum MyErr {
    Diesel(diesel::result::Error),
    TooManySortColumns(String),
    UpdateError(String)
}

impl Error for MyErr {
    fn description(&self) -> &str {
        match *self {
            MyErr::Diesel(ref err) => err.description(),
            MyErr::UpdateError(ref err) => err,
            MyErr::TooManySortColumns(ref err) => err
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            MyErr::Diesel(ref err) => err.cause(),
            MyErr::UpdateError(_) | MyErr::TooManySortColumns(_) => None
        }
    }
}

impl Display for MyErr {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            MyErr::Diesel(ref err) => err.fmt(f),
            MyErr::UpdateError(ref err) => err.fmt(f),
            MyErr::TooManySortColumns(ref err) => err.fmt(f)
        }
    }
}
