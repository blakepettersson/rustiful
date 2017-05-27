extern crate iron;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use self::iron::prelude::*;
use super::errors::BodyParserError;
use super::super::RequestResult;
use FromRequest;
use errors::FromRequestError;
use errors::RequestError;
use object::JsonApiObject;
use request::FromPost;
use serde::Deserialize;
use service::JsonPost;
use status::Status;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryInto;
use try_from::TryFrom;
use sort_order::SortOrder;
use errors::QueryStringParseError;

autoimpl! {
    pub trait PostHandler<'a, T> where
        T: JsonPost + ToJson + FromPost<'a, T>,
        T::Error: 'static,
        <T::Context as FromRequest>::Error: 'static,
        Status: for<'b> From<&'b T::Error>,
        T::Attrs: 'static + for<'b> Deserialize<'b>,
        T::SortField: for<'b> TryFrom<(&'b str, SortOrder), Error = QueryStringParseError>,
        T::FilterField: for<'b> TryFrom<(&'b str, Vec<&'b str>), Error = QueryStringParseError>,
        <T::JsonApiIdType as FromStr>::Err: Error
    {
        fn post(req: &'a mut Request) -> IronResult<Response> {
            match req.get::<bodyparser::Struct<JsonApiObject<T::Attrs>>>() {
                Ok(Some(post)) => {
                    match FromRequest::from_request(req) {
                        Ok(res) => {
                            let query = req.url.query().unwrap_or("");
                            let result = <T as FromPost<T>>::create(query, post.data, res);
                            RequestResult(result, Status::Created).try_into()
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
