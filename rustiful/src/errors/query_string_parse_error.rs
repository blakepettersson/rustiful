use std::error::Error;
use std::fmt::*;

static UNIMPLEMENTED: &'static str = "Unimplemented";

#[derive(Debug, PartialEq, Eq)]
pub enum QueryStringParseError {
    InvalidParam(String),
    InvalidKeyParam(String),
    InvalidValue(String),
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
        DuplicateSortKey(ref desc) => desc,
        UnImplementedError => UNIMPLEMENTED,
    }
}
