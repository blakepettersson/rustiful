extern crate iron;

use self::iron::prelude::*;
use super::status::Status;
use std;

/// A trait used to initialize a type from an Iron request.
///
/// # Example
///
/// ```
/// # extern crate iron;
/// # extern crate rustiful;
/// #
/// # use std::error::Error;
/// # use std::fmt::Display;
/// # use std::fmt::Formatter;
/// # use rustiful::iron::FromRequest;
/// # use rustiful::iron::status::Status;
/// #
/// struct Foo {
///     magic_header: String
/// }
///
/// #[derive(Debug)]
/// struct FooError(String);
/// #
/// # impl Error for FooError {
/// #     fn description(&self) -> &str {
/// #        &self.0
/// #   }
/// #
/// #   fn cause(&self) -> Option<&Error> {
/// #       None
/// #   }
/// # }
/// #
/// # impl Display for FooError {
/// #     fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
/// #        self.0.fmt(f)
/// #    }
/// # }
///
/// impl FromRequest for Foo {
///     type Error = FooError;
///
///     fn from_request(request: &iron::Request) -> Result<Self, (Self::Error, Status)> {
///         let magic_header = request.headers.get_raw("my-magic-header");
///
///         if magic_header == None {
///             Err((FooError("header not present!".to_string()), Status::BadRequest))
///         } else {
///             // Get first header value if present.. there are better ways to do this, but this is
///             // for demo purposes only
///             let magic_header_value = magic_header.unwrap().first().unwrap().to_vec();
///             Ok(Foo {
///                 magic_header: String::from_utf8(magic_header_value).unwrap()
///             })
///         }
///     }
/// }
/// #
/// # fn main() {}
/// ```
pub trait FromRequest: Sized {
    type Error: std::error::Error + Send;

    fn from_request(request: &Request) -> Result<Self, (Self::Error, Status)>;
}
