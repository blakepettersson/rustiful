extern crate iron;

use self::iron::prelude::*;
use super::Status;
use super::super::json_api_type;
use FromRequest;
use errors::FromRequestError;
use errors::IdParseError;
use errors::RepositoryError;
use iron::id;
use service::JsonDelete;
use std::error::Error;
use std::str::FromStr;

pub trait DeleteHandler
    where Self: JsonDelete
{
    fn respond<'r>(req: &'r mut Request) -> IronResult<Response>
        where Status: for<'b> From<&'b Self::Error>,
              <Self::JsonApiIdType as FromStr>::Err: Error
    {
        let ctx = match <Self::Context as FromRequest>::from_request(req) {
            Ok(result) => result,
            Err(e) => {
                return FromRequestError::<<Self::Context as FromRequest>::Error>(e).into()
            }
        };

        let id = match <Self::JsonApiIdType>::from_str(id(req)) {
            Ok(result) => result,
            Err(e) => return IdParseError(e).into(),
        };

        match Self::delete(id, ctx) {
            Ok(_) => Ok(Response::with((json_api_type(), Status::NoContent))),
            Err(e) => RepositoryError::new(e).into(),
        }
    }
}

impl<T> DeleteHandler for T where T: JsonDelete {}
