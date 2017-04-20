extern crate iron;

use self::iron::prelude::*;
use std;

pub trait FromRequest: Sized {
    type Error: std::error::Error + Send;

    fn from_request(request: &Request) -> Result<Self, Self::Error>;
}
