use std::fmt::*;
use std::error::Error;
use id::JsonApiId;
use serde::ser::Serialize;
use serde::de::Deserialize;

pub trait ToJson {
    type Attrs: Serialize + Deserialize + Debug;
    type Resource: Serialize + Deserialize + Debug;

    fn id(&self) -> JsonApiId;

    fn type_name(&self) -> String;
}

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
