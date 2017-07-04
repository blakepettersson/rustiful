extern crate iron;

use self::iron::prelude::*;
use super::Status;
use super::super::FromRequest;
use super::super::JsonErrorResponse;
use super::super::json_api_type;
use errors::IdParseError;
use iron::id;
use service::Handler;
use service::JsonDelete;
use std::error::Error;
use std::str::FromStr;

pub trait DeleteHandler
where
    Self: JsonDelete
{
    fn respond<'r>(req: &'r mut Request) -> IronResult<Response>
    where
        Self: Handler<Status = Status>,
        Self::Context: FromRequest,
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

        match Self::delete(id, ctx) {
            Ok(_) => Ok(Response::with((json_api_type(), Status::NoContent))),
            Err((e, status)) => JsonErrorResponse(e, status).into()
        }
    }
}

impl<T: JsonDelete> DeleteHandler for T {}
