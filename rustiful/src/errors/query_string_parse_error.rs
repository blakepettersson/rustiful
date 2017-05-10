use std::error::Error;
use std::fmt::*;

static UNIMPLEMENTED: &'static str = "Unimplemented";

#[derive(Debug, PartialEq, Eq)]
/// All types of errors that can happen when attempting to parse a query string.
pub enum QueryStringParseError {
    EmptyKey(String),
    EmptyValue(String),
    InvalidParam(String),
    InvalidKeyParam(String),
    InvalidValue(String),
    EmptyFieldsetKey(String),
    EmptyFieldsetValue(String),
    ParseError(String),
    DuplicateSortKey(String),
    UnImplementedError,
}

impl Display for QueryStringParseError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use self::QueryStringParseError::*;

        let msg = "Query string parse error: ";
        match *self {
            InvalidParam(ref desc) => write!(f, "{} Invalid param: {}", msg, desc),
            InvalidKeyParam(ref desc) => write!(f, "{} Invalid key: {}", msg, desc),
            InvalidValue(ref desc) => write!(f, "{} Invalid value: {}", msg, desc),
            ParseError(ref desc) => write!(f, "{} Parse error: {}", msg, desc),
            EmptyKey(ref desc) => write!(f, "{} Empty key in query parameter: {}", msg, desc),
            EmptyValue(ref desc) => write!(f, "{} Empty value in query parameter: {}", msg, desc),
            EmptyFieldsetKey(ref desc) => write!(f, "{} No field type specified for {}", msg, desc),
            EmptyFieldsetValue(ref desc) => {
                write!(f, "{} No values specified for fields[{}]", msg, desc)
            },
            DuplicateSortKey(ref desc) => write!(f, "{} Duplicate sort param key: {}", msg, desc),
            UnImplementedError => write!(f, "{} Unimplemented!", msg),
        }
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

fn description(error: &QueryStringParseError) -> &str {
    use self::QueryStringParseError::*;

    match *error {
        InvalidParam(ref desc) => desc,
        InvalidKeyParam(ref desc) => desc,
        InvalidValue(ref desc) => desc,
        ParseError(ref desc) => desc,
        EmptyKey(ref desc) => desc,
        EmptyValue(ref desc) => desc,
        EmptyFieldsetKey(ref desc) => desc,
        EmptyFieldsetValue(ref desc) => desc,
        DuplicateSortKey(ref desc) => desc,
        UnImplementedError => UNIMPLEMENTED,
    }
}
