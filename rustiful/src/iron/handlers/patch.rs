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
use iron::id;
use object::JsonApiObject;
use params::TypedParams;
use request::FromPatch;
use serde::Deserialize;
use service::JsonPatch;
use sort_order::SortOrder;
use status::Status;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;
use try_from::TryInto;

autoimpl! {
    pub trait PatchHandler<'a, T> where
        T: JsonPatch + ToJson + FromPatch<'a, T>,
        T::Error: 'static,
        Status: for<'b> From<&'b T::Error>,
        <T::Context as FromRequest>::Error: 'static,
        T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
        T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
        T::Params: TypedParams<T::SortField, T::FilterField> + Default,
        T::Attrs: for<'b> From<(T, &'b T::Params)> + 'static + for<'b> Deserialize<'b>,
        <T::JsonApiIdType as FromStr>::Err: Error
    {
        fn patch(req: &'a mut Request) -> IronResult<Response> {
            match req.get::<bodyparser::Struct<JsonApiObject<T::Attrs>>>() {
                Ok(Some(patch)) => {
                    match FromRequest::from_request(req) {
                        Ok(res) => {
                            let result = <T as FromPatch<T>>::patch(id(req), patch.data, res);
                            RequestResult(result, Status::Ok).try_into()
                        },
                        Err(e) => FromRequestError::<<T::Context as FromRequest>::Error>(e).into()
                    }
                },
                Ok(None) => {
                    let err:RequestError<T::Error, T::JsonApiIdType> = RequestError::NoBody;
                    err.into()
                },
                Err(e) => BodyParserError(e).into()
            }
        }
    }
}
