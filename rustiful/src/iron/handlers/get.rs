extern crate iron;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use self::iron::prelude::*;
use super::super::RequestResult;
use FromRequest;
use errors::FromRequestError;
use errors::IdParseError;
use errors::QueryStringParseError;
use errors::RequestError;
use iron::id;
use request::get::get;
use service::JsonGet;
use params::SortOrder;
use status::Status;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;
use try_from::TryInto;

autoimpl! {
    pub trait GetHandler<'a, T> where
        T: JsonGet + ToJson,
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

            let id = match <T::JsonApiIdType>::from_str(id(req)) {
                Ok(result) => result,
                Err(e) => {
                    return RequestError::IdParseError::<T::Error, T::JsonApiIdType>(IdParseError(e)).into()
                }
            };

            let result = get::<T>(id, req.url.query().unwrap_or(""), ctx);
            RequestResult(result, Status::Ok).try_into()
        }
    }
}
