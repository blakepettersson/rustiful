extern crate iron;
extern crate bodyparser;

use self::iron::prelude::*;
use super::errors::BodyParserError;
use super::super::into_json_api_response;
use super::super::FromRequest;
use super::super::status::Status;
use container::JsonApiContainer;
use data::JsonApiData;
use errors::FromRequestError;
use errors::IdParseError;
use errors::QueryStringParseError;
use errors::RepositoryError;
use errors::RequestError;
use iron::id;
use params::SortOrder;
use service::JsonPatch;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;

pub trait PatchHandler
    where Self: JsonPatch
{
    fn respond<'r>(req: &'r mut Request) -> IronResult<Response>
        where Status: for<'b> From<&'b Self::Error>,
              Self: 'static,
              Self: ToJson,
              Self::Context: FromRequest,
              Self::SortField: TryFrom<(&'r str, SortOrder), Error = QueryStringParseError>,
              Self::FilterField: TryFrom<(&'r str, Vec<&'r str>), Error = QueryStringParseError>,
              <Self::JsonApiIdType as FromStr>::Err: Error
    {
        let json = match req.get::<bodyparser::Struct<JsonApiContainer<JsonApiData<Self>>>>() {
            Ok(Some(patch)) => patch,
            Ok(None) => return RequestError::NoBody.into(),
            Err(e) => return BodyParserError(e).into(),
        };

        let ctx = match <Self::Context as FromRequest>::from_request(req) {
            Ok(result) => result,
            Err(e) => {
                return FromRequestError::<<Self::Context as FromRequest>::Error>(e).into()
            }
        };

        let id = match <Self::JsonApiIdType>::from_str(id(req)) {
            Ok(result) => result,
            Err(e) => return IdParseError(e).into(),
        };

        let params = match Self::Params::from_str(req.url.query().unwrap_or("")) {
            Ok(result) => result,
            Err(e) => return e.into(),
        };

        match Self::update(id, json.data, &params, ctx) {
            Ok(result) => into_json_api_response(result, Status::Ok),
            Err(e) => RepositoryError::new(e).into(),
        }
    }
}

impl<T> PatchHandler for T where T: JsonPatch {}
