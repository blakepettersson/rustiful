extern crate iron;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use self::iron::prelude::*;
use super::super::into_json_api_response;
use FromRequest;
use errors::FromRequestError;
use errors::IdParseError;
use errors::QueryStringParseError;
use errors::RepositoryError;
use errors::RequestError;
use iron::id;
use params::SortOrder;
use service::JsonGet;
use status::Status;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;

pub trait GetHandler
    where Self: JsonGet
{
    fn respond<'r>(req: &'r mut Request) -> IronResult<Response>
        where Status: for<'b> From<&'b Self::Error>,
              Self: ToJson,
              Self::SortField: TryFrom<(&'r str, SortOrder), Error = QueryStringParseError>,
              Self::FilterField: TryFrom<(&'r str, Vec<&'r str>), Error = QueryStringParseError>,
              <Self::JsonApiIdType as FromStr>::Err: Error
    {
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

        match Self::find(id, &params, ctx) {
            Ok(Some(result)) => into_json_api_response(result, Status::Ok),
            Ok(None) => RequestError::NotFound.into(),
            Err(e) => RepositoryError::new(e).into(),
        }
    }
}

impl<T> GetHandler for T where T: JsonGet {}
