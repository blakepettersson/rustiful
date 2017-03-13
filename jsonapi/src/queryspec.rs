use std::fmt::*;
use std::str::FromStr;
use std::error::Error;

pub trait ToParams {
    type Params: FromStr;
}

pub trait ToSortFields {
    type SortField;
}

#[derive(Debug, PartialEq, Eq)]
pub enum QueryStringParseError {
    InvalidParam(String),
    InvalidKeyParam(String),
    InvalidValue(String),
    ParseError(String),
    UnImplementedError,
}

static UNIMPLEMENTED: &'static str = "Unimplemented";

impl Display for QueryStringParseError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use self::QueryStringParseError::*;

        let msg = match *self {
            InvalidParam(ref desc) => desc,
            InvalidKeyParam(ref desc) => desc,
            InvalidValue(ref desc) => desc,
            ParseError(ref desc) => desc,
            UnImplementedError => UNIMPLEMENTED,
        };
        write!(f, "Query string parse error: {}", msg)
    }
}

impl Error for QueryStringParseError {
    fn description(&self) -> &str {
        use self::QueryStringParseError::*;

        match *self {
            InvalidParam(ref desc) => desc,
            InvalidKeyParam(ref desc) => desc,
            InvalidValue(ref desc) => desc,
            ParseError(ref desc) => desc,
            UnImplementedError => UNIMPLEMENTED,
        }
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
