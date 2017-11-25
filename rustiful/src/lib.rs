#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
#![warn(missing_debug_implementations, missing_copy_implementations, trivial_casts,
        trivial_numeric_casts, unused_import_braces, unused_qualifications)]

extern crate serde;

mod json;

mod to_json;
pub use to_json::*;

mod service;
pub use service::*;

mod data;
pub use data::*;

mod try_from;
pub use try_from::*;

mod params;
pub use params::*;

mod errors;
pub use errors::query_string_parse_error::QueryStringParseError;

mod container;
pub use container::*;

mod error;
pub use error::*;

mod builder;
pub use builder::*;

mod resource;
pub use resource::*;

#[cfg(feature = "iron")]
pub mod iron;

#[macro_use]
extern crate serde_derive;

pub mod json_option;

#[cfg(feature = "rustiful-derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate rustiful_derive;
#[cfg(feature = "rustiful-derive")]
#[doc(hidden)]
pub use rustiful_derive::*;
