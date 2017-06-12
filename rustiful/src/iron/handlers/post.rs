extern crate iron;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use self::iron::prelude::*;
use super::errors::BodyParserError;
use super::super::into_json_api_response;
use FromRequest;
use container::JsonApiContainer;
use data::JsonApiData;
use errors::FromRequestError;
use errors::QueryStringParseError;
use errors::RepositoryError;
use errors::RequestError;
use params::SortOrder;
use serde::Deserialize;
use service::JsonPost;
use status::Status;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;

autoimpl! {
    pub trait PostHandler<'a, T> where
        T: 'static + JsonPost + ToJson,
        T::Error: 'static,
        <T::Context as FromRequest>::Error: 'static,
        Status: for<'b> From<&'b T::Error>,
        T::Attrs: 'static + for<'b> Deserialize<'b>,
        T::SortField: for<'b> TryFrom<(&'b str, SortOrder), Error = QueryStringParseError>,
        T::FilterField: for<'b> TryFrom<(&'b str, Vec<&'b str>), Error = QueryStringParseError>,
        <T::JsonApiIdType as FromStr>::Err: Error
    {
        fn respond(req: &'a mut Request) -> IronResult<Response> {
            let json = match req.get::<bodyparser::Struct<JsonApiContainer<JsonApiData<T>>>>() {
                Ok(Some(patch)) => patch,
                Ok(None) => return RequestError::NoBody.into(),
                Err(e) => return BodyParserError(e).into()
            };

            let ctx = match <T::Context as FromRequest>::from_request(req) {
                Ok(result) => result,
                Err(e) => return FromRequestError::<<T::Context as FromRequest>::Error>(e).into()
            };

            let params = match T::Params::from_str(req.url.query().unwrap_or("")) {
                Ok(result) => result,
                Err(e) => return e.into()
            };

            match T::create(json.data, &params, ctx) {
                Ok(result) => into_json_api_response(result, Status::Ok),
                Err(e) => RepositoryError::new(e).into()
            }
        }
    }
}
