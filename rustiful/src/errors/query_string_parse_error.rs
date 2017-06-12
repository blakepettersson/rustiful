use std::error::Error;
use std::fmt::*;

static UNIMPLEMENTED: &'static str = "Unimplemented";

#[derive(Debug, PartialEq, Eq)]
/// All types of errors that can happen when attempting to parse a query string.
pub enum QueryStringParseError {
    /// `fields` query param is in an invalid format
    InvalidFieldsetKey(String),

    /// `fields` value does not match field name
    InvalidFieldValue(String),

    /// `sort` value does not match field name
    InvalidSortValue(String),

    /// No `fields[*]` values specified in value
    EmptyFieldsetValue(String),

    /// Multiple `sort` query param keys, e.g `sort=foo&sort=bar`
    DuplicateSortKey(String),

    /// Currently unsupported functionality when parsing the query param, notably relationships
    UnImplementedError,
}

impl Display for QueryStringParseError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use self::QueryStringParseError::*;

        let msg = "Query string parse error:";
        match *self {
            InvalidFieldsetKey(ref desc) => write!(f, "{} Invalid key: {}", msg, desc),
            InvalidFieldValue(ref desc) => write!(f, "{} Invalid value: {}", msg, desc),
            InvalidSortValue(ref desc) => write!(f, "{} Invalid value: {}", msg, desc),
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
        InvalidFieldValue(ref desc) => desc,
        InvalidSortValue(ref desc) => desc,
        InvalidFieldsetKey(ref desc) => desc,
        EmptyFieldsetValue(ref desc) => desc,
        DuplicateSortKey(ref desc) => desc,
        UnImplementedError => UNIMPLEMENTED,
    }
}
