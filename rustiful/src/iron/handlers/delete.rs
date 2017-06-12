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

autoimpl! {
    pub trait DeleteHandler<'a, T>
        where T: JsonDelete,
              T::Error: 'static,
              Status: for<'b> From<&'b T::Error>,
              <T::JsonApiIdType as FromStr>::Err: Error
    {
        fn respond(req: &'a mut Request) -> IronResult<Response> {
            let ctx = match <T::Context as FromRequest>::from_request(req) {
                Ok(result) => result,
                Err(e) => return FromRequestError::<<T::Context as FromRequest>::Error>(e).into()
            };

            let id = match <T::JsonApiIdType>::from_str(id(req)) {
                Ok(result) => result,
                Err(e) => return IdParseError(e).into()
            };

            match T::delete(id, ctx) {
                Ok(_) => Ok(Response::with((json_api_type(), Status::NoContent))),
                Err(e) => RepositoryError::new(e).into()
            }
        }
    }
}
