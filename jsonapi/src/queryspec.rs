extern crate serde;

use std::str::FromStr;

use std::marker::PhantomData;
use std::collections::HashMap;
use std::collections::HashSet;
use self::serde::ser::Serialize;
use self::serde::de::Deserialize;

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
