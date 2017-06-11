extern crate iron;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use self::iron::prelude::*;
use super::errors::BodyParserError;
use super::super::RequestResult;
use FromRequest;
use errors::FromRequestError;
use errors::IdParseError;
use errors::QueryStringParseError;
use errors::RequestError;
use iron::id;
use request::patch::patch;
use serde::Deserialize;
use service::JsonPatch;
use params::SortOrder;
use status::Status;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;
use try_from::TryInto;
use container::JsonApiContainer;
use data::JsonApiData;

autoimpl! {
    pub trait PatchHandler<'a, T> where
        T: 'static + JsonPatch + ToJson,
        T::Error: 'static,
        Status: for<'b> From<&'b T::Error>,
        T::Attrs: 'static + for<'b> Deserialize<'b>,
        T::SortField: for<'b> TryFrom<(&'b str, SortOrder), Error = QueryStringParseError>,
        T::FilterField: for<'b> TryFrom<(&'b str, Vec<&'b str>), Error = QueryStringParseError>,
        <T::JsonApiIdType as FromStr>::Err: Error
    {
        fn respond(req: &'a mut Request) -> IronResult<Response> {
            let json = match req.get::<bodyparser::Struct<JsonApiContainer<JsonApiData<T>>>>() {
                Ok(Some(patch)) => patch,
                Ok(None) => {
                    let err:RequestError<T::Error> = RequestError::NoBody;
                    return err.into()
                },
                Err(e) => return BodyParserError(e).into()
            };

            let ctx = match <T::Context as FromRequest>::from_request(req) {
                Ok(result) => result,
                Err(e) => return FromRequestError::<<T::Context as FromRequest>::Error>(e).into()
            };

            let id = match <T::JsonApiIdType>::from_str(id(req)) {
                Ok(result) => result,
                Err(e) => return IdParseError(e).into()
            };

            let result = patch::<T>(id, req.url.query().unwrap_or(""), json.data, ctx);
            RequestResult(result, Status::Ok).try_into()
        }
    }
}
