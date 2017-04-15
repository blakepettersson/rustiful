extern crate iron;

use self::iron::prelude::*;
use super::Status;
use super::super::RequestResult;
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
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {
        fn delete(req: &'a mut Request) -> IronResult<Response> {
            let result = <T as FromDelete<T>>::delete(id(req), Default::default());
            RequestResult(result, Status::NoContent).try_into()
        }
    }
}
