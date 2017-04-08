extern crate iron;

use std::error::Error;
use std::str::FromStr;
use self::iron::prelude::*;
use service::JsonDelete;
use request::FromDelete;
use iron::id;
use super::Status;
use try_from::TryInto;
use super::super::RequestResult;

autoimpl! {
    pub trait DeleteHandler<'a, T>
        where T: JsonDelete + FromDelete<'a, T>,
              T::Error: 'static,
              Status: for<'b> From<&'b T::Error>,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {
        fn delete(req: &'a mut Request) -> IronResult<Response> {
            let result = <T as FromDelete<T>>::delete(id(req), Default::default());
            RequestResult(result, Status::NoContent).try_into()
        }
    }
}
