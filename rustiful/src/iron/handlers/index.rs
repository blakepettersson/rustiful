extern crate iron;

use self::iron::prelude::*;
use super::super::FromRequest;
use super::super::JsonErrorResponse;
use super::super::JsonOkResponse;
use super::super::status::Status;
use errors::QueryStringParseError;
use params::SortOrder;
use service::Handler;
use service::JsonIndex;
use std::str::FromStr;
use to_json::ToJson;
use std::convert::TryFrom;

pub trait IndexHandler
where
    Self: JsonIndex
{
    fn respond<'r>(req: &'r mut Request) -> IronResult<Response>
    where
        Self: Handler<Status = Status>,
        Self: ToJson,
        Self::Context: FromRequest,
    {
        let ctx = match <Self::Context as FromRequest>::from_request(req) {
            Ok(result) => result,
            Err((e, status)) => return JsonErrorResponse(e, status).into()
        };

        let params = match Self::Params::from_str(req.url.query().unwrap_or("")) {
            Ok(result) => result,
            Err(e) => return JsonErrorResponse(e, Status::BadRequest).into()
        };

        match Self::find_all(&params, ctx) {
            Ok(result) => JsonOkResponse(result).into(),
            Err((e, status)) => JsonErrorResponse(e, status).into()
        }
    }
}

impl<T: JsonIndex> IndexHandler for T {}
