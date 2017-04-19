extern crate iron;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use self::iron::prelude::*;
use self::iron::status;
use super::super::RequestResult;
use errors::QueryStringParseError;
use errors::RequestError;
use params::TypedParams;
use request::FromPost;
use serde::Deserialize;
use serde::Serialize;
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
        T::Context: Default,
        Status: for<'b> From<&'b T::Error>,
        T::Resource: Serialize + Deserialize + Clone + 'static + for<'b> From<(T, &'b T::Params)>,
        T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
        T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
        T::Params: TypedParams<T::SortField, T::FilterField> + Default,
        T::Attrs: for<'b> From<(T, &'b T::Params)>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {
        fn post(req: &'a mut Request) -> IronResult<Response> {
            match req.get::<bodyparser::Struct<T::Resource>>() {
                Ok(Some(post)) => {
                    let result = <T as FromPost<T>>::create(post, Default::default());
                    RequestResult(result, Status::Created).try_into()
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
