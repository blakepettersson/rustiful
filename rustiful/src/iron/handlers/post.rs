extern crate iron;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use self::iron::prelude::*;
use super::errors::BodyParserError;
use super::super::RequestResult;
use FromRequest;
use errors::FromRequestError;
use errors::QueryStringParseError;
use errors::RequestError;
use object::JsonApiObject;
use request::post::post;
use serde::Deserialize;
use service::JsonPost;
use params::SortOrder;
use status::Status;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;
use try_from::TryInto;

autoimpl! {
    pub trait PostHandler<'a, T> where
        T: JsonPost + ToJson,
        T::Error: 'static,
        <T::Context as FromRequest>::Error: 'static,
        Status: for<'b> From<&'b T::Error>,
        T::Attrs: 'static + for<'b> Deserialize<'b>,
        T::SortField: for<'b> TryFrom<(&'b str, SortOrder), Error = QueryStringParseError>,
        T::FilterField: for<'b> TryFrom<(&'b str, Vec<&'b str>), Error = QueryStringParseError>,
        <T::JsonApiIdType as FromStr>::Err: Error
    {
        fn respond(req: &'a mut Request) -> IronResult<Response> {
            let json = match req.get::<bodyparser::Struct<JsonApiObject<T::Attrs>>>() {
                Ok(Some(patch)) => patch,
                Ok(None) => {
                    let err:RequestError<T::Error, T::JsonApiIdType> = RequestError::NoBody;
                    return err.into()
                },
                Err(e) => return BodyParserError(e).into()
            };

            let ctx = match <T::Context as FromRequest>::from_request(req) {
                Ok(result) => result,
                Err(e) => return FromRequestError::<<T::Context as FromRequest>::Error>(e).into()
            };

            let result = post::<T>(req.url.query().unwrap_or(""), json.data, ctx);
            RequestResult(result, Status::Created).try_into()
        }
    }
}
