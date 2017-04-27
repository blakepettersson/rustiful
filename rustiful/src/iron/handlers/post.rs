extern crate iron;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use self::iron::prelude::*;
use self::iron::status;
use super::super::RequestResult;
use ::FromRequest;
use errors::QueryStringParseError;
use errors::RequestError;
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
use object::JsonApiObject;

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
                        Err(e) => Err(IronError::new(e, Status::InternalServerError))
                    }
                },
                Ok(None) => {
                    let err:RequestError<T::Error> = RequestError::NoBody;
                    Err(IronError::new(err, status::InternalServerError))
                },
                Err(e) => Err(IronError::new(e, status::InternalServerError))
            }
        }
    }
}
