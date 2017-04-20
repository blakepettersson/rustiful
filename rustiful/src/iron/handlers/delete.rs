extern crate iron;

use self::iron::prelude::*;
use super::Status;
use super::super::RequestResult;
use ::FromRequest;
use iron::id;
use request::FromDelete;
use service::JsonDelete;
use std::error::Error;
use std::str::FromStr;
use try_from::TryInto;

autoimpl! {
    pub trait DeleteHandler<'a, T>
        where T: JsonDelete + FromDelete<'a, T>,
              T::Error: 'static,
              Status: for<'b> From<&'b T::Error>,
              <T::Context as FromRequest>::Error: 'static,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {
        fn delete(req: &'a mut Request) -> IronResult<Response> {
            match FromRequest::from_request(req) {
                Ok(res) => {
                    let result = <T as FromDelete<T>>::delete(id(req), res);
                    RequestResult(result, Status::NoContent).try_into()
                },
                Err(e) => Err(IronError::new(e, Status::InternalServerError))
            }
        }
    }
}
