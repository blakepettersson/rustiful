extern crate iron;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use self::iron::prelude::*;
use super::super::RequestResult;
use FromRequest;
use errors::FromRequestError;
use errors::QueryStringParseError;
use request::index::index;
use service::JsonIndex;
use sort_order::SortOrder;
use status::Status;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;
use try_from::TryInto;

autoimpl! {
    pub trait IndexHandler<'a, T> where
        T: JsonIndex + ToJson,
        T::Error: 'static,
        Status: for<'b> From<&'b T::Error>,
        T::SortField: for<'b> TryFrom<(&'b str, SortOrder), Error = QueryStringParseError>,
        T::FilterField: for<'b> TryFrom<(&'b str, Vec<&'b str>), Error = QueryStringParseError>,
        <T::JsonApiIdType as FromStr>::Err: Error
    {
        fn respond(req: &'a mut Request) -> IronResult<Response> {
            let ctx = match <T::Context as FromRequest>::from_request(req) {
                Ok(result) => result,
                Err(e) => return FromRequestError::<<T::Context as FromRequest>::Error>(e).into()
            };

            let result = index::<T>(req.url.query().unwrap_or(""), ctx);
            RequestResult(result, Status::Ok).try_into()
        }
    }
}
