extern crate iron;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use self::iron::prelude::*;
use super::super::FromRequest;
use super::super::JsonErrorResponse;
use super::super::JsonOkResponse;
use super::super::status::Status;
use errors::IdParseError;
use errors::QueryStringParseError;
use errors::RequestError;
use iron::id;
use params::SortOrder;
use service::Handler;
use service::JsonGet;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;

pub trait GetHandler
where
    Self: JsonGet
{
    fn respond<'r>(req: &'r mut Request) -> IronResult<Response>
    where
        Self: ToJson,
        Self: Handler<Status = Status>,
        Self::Context: FromRequest,
        Self::SortField: TryFrom<(&'r str, SortOrder), Error = QueryStringParseError>,
        Self::FilterField: TryFrom<(&'r str, Vec<&'r str>), Error = QueryStringParseError>,
        <Self::JsonApiIdType as FromStr>::Err: Error
    {
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

        match Self::find(id, &params, ctx) {
            Ok(Some(result)) => JsonOkResponse(result).into(),
            Ok(None) => JsonErrorResponse(RequestError::NotFound, Status::BadRequest).into(),
            Err((e, status)) => JsonErrorResponse(e, status).into()
        }
    }
}

impl<T: JsonGet> GetHandler for T {}
