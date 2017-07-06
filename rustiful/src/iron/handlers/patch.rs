extern crate iron;
extern crate bodyparser;

use self::iron::prelude::*;
use super::super::FromRequest;
use super::super::JsonErrorResponse;
use super::super::JsonOkResponse;
use super::super::status::Status;
use container::JsonApiContainer;
use data::JsonApiData;
use errors::IdParseError;
use errors::QueryStringParseError;
use errors::RequestError;
use iron::id;
use params::SortOrder;
use service::Handler;
use service::JsonPatch;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;

pub trait PatchHandler
where
    Self: JsonPatch
{
    fn respond<'r>(req: &'r mut Request) -> IronResult<Response>
    where
        Self: Handler<Status = Status>,
        Self: 'static,
        Self: ToJson,
        Self::Context: FromRequest,
        Self::SortField: TryFrom<(&'r str, SortOrder), Error = QueryStringParseError>,
        Self::FilterField: TryFrom<(&'r str, Vec<&'r str>), Error = QueryStringParseError>,
        <Self::JsonApiIdType as FromStr>::Err: Error
    {
        let json = match req.get::<bodyparser::Struct<JsonApiContainer<JsonApiData<Self>>>>() {
            Ok(Some(patch)) => patch,
            Ok(None) => return JsonErrorResponse(RequestError::NoBody, Status::BadRequest).into(),
            Err(e) => return JsonErrorResponse(e, Status::BadRequest).into()
        };

        let ctx = match <Self::Context as FromRequest>::from_request(req) {
            Ok(result) => result,
            Err((e, status)) => return JsonErrorResponse(e, status).into()
        };

        let id = match <Self::JsonApiIdType>::from_str(id(req)) {
            Ok(result) => result,
            Err(e) => return JsonErrorResponse(IdParseError(e), Status::BadRequest).into()
        };

        let params = match Self::Params::from_str(req.url.query().unwrap_or("")) {
            Ok(result) => result,
            Err(e) => return JsonErrorResponse(e, Status::BadRequest).into()
        };

        match Self::update(id, json.data, &params, ctx) {
            Ok(result) => JsonOkResponse(result).into(),
            Err((e, status)) => JsonErrorResponse(e, status).into()
        }
    }
}

impl<T: JsonPatch> PatchHandler for T {}
