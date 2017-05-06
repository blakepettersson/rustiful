extern crate iron;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use self::iron::prelude::*;
use super::errors::BodyParserError;
use super::super::RequestResult;
use FromRequest;
use errors::QueryStringParseError;
use errors::RequestError;
use errors::FromRequestError;
use object::JsonApiObject;
use params::TypedParams;
use request::FromPost;
use serde::Deserialize;
use service::JsonPost;
use sort_order::SortOrder;
use status::Status;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;
use try_from::TryInto;

autoimpl! {
    pub trait PostHandler<'a, T> where
        T: JsonPost + ToJson + FromPost<'a, T>,
        T::Error: 'static,
        <T::Context as FromRequest>::Error: 'static,
        Status: for<'b> From<&'b T::Error>,
        T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
        T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
        T::Params: TypedParams<T::SortField, T::FilterField> + Default,
        T::Attrs: for<'b> From<(T, &'b T::Params)> + 'static + for<'b> Deserialize<'b>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {
        fn post(req: &'a mut Request) -> IronResult<Response> {
            match req.get::<bodyparser::Struct<JsonApiObject<T::Attrs>>>() {
                Ok(Some(post)) => {
                    match FromRequest::from_request(req) {
                        Ok(res) => {
                            let result = <T as FromPost<T>>::create(post.data, res);
                            RequestResult(result, Status::Created).try_into()
                        },
                        Err(e) => FromRequestError::<<T::Context as FromRequest>::Error>(e).into()
                    }
                },
                Ok(None) => {
                    let err:RequestError<T::Error> = RequestError::NoBody;
                    err.into()
                },
                Err(e) => BodyParserError(e).into()
            }
        }
    }
}
