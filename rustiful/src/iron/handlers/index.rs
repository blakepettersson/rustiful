extern crate iron;

use self::iron::prelude::*;
use super::super::into_json_api_response;
use super::super::FromRequest;
use errors::FromRequestError;
use errors::QueryStringParseError;
use errors::RepositoryError;
use params::SortOrder;
use service::JsonIndex;
use status::Status;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;

pub trait IndexHandler
    where Self: JsonIndex
{
    fn respond<'r>(req: &'r mut Request) -> IronResult<Response>
        where Status: for<'b> From<&'b Self::Error>,
              Self: ToJson,
              Self::Context: FromRequest,
              Self::SortField: TryFrom<(&'r str, SortOrder), Error = QueryStringParseError>,
              Self::FilterField: TryFrom<(&'r str, Vec<&'r str>), Error = QueryStringParseError>
    {
        let ctx = match <Self::Context as FromRequest>::from_request(req) {
            Ok(result) => result,
            Err(e) => {
                return FromRequestError::<<Self::Context as FromRequest>::Error>(e).into()
            }
        };

        let params = match Self::Params::from_str(req.url.query().unwrap_or("")) {
            Ok(result) => result,
            Err(e) => return e.into(),
        };

        match Self::find_all(&params, ctx) {
            Ok(result) => into_json_api_response(result, Status::Ok),
            Err(e) => RepositoryError::new(e).into(),
        }
    }
}

impl<T> IndexHandler for T where T: JsonIndex {}
