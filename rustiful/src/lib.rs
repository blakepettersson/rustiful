#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
#![warn(missing_debug_implementations, missing_copy_implementations, trivial_casts,
trivial_numeric_casts, unused_import_braces, unused_qualifications)]

extern crate serde;

mod to_json;
pub use to_json::*;

mod service;
pub use service::*;

mod sort_order;
pub use sort_order::*;

mod data;
pub use data::*;

mod try_from;
pub use try_from::*;

mod params;
pub use params::*;

mod errors;
pub use errors::QueryStringParseError;

mod array;
pub use array::*;

mod object;
pub use object::*;

mod error;
pub use error::*;

mod request;

#[cfg(feature = "iron")]
pub mod iron;

#[cfg(feature = "iron")]
pub use iron::from_request::*;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate autoimpl;

extern crate hyper;

/// Status Codes
pub mod status {
    pub use hyper::status::StatusClass;
    pub use hyper::status::StatusCode as Status;
    pub use hyper::status::StatusCode::*;
}

pub mod json_option;
