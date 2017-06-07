extern crate iron;

use self::iron::prelude::*;
use super::Status;
use super::super::RequestResult;
use FromRequest;
use errors::FromRequestError;
use errors::IdParseError;
use errors::RequestError;
use request::delete::delete;
use iron::id;
use service::JsonDelete;
use std::error::Error;
use std::str::FromStr;
use try_from::TryInto;

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
                Err(e) => {
                    return RequestError::IdParseError::<T::Error, T::JsonApiIdType>(IdParseError(e)).into()
                }
            };

            let result = delete::<T>(id, ctx);
            RequestResult(result, Status::NoContent).try_into()
        }
    }
}
