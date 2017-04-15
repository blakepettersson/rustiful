extern crate iron;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use self::iron::prelude::*;
use self::iron::status;
use super::super::RequestResult;
use errors::QueryStringParseError;
use errors::RequestError;
use iron::id;
use params::TypedParams;
use request::FromPatch;
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
        T::Resource: 'static + for<'b> From<(T, &'b T::Params)>,
        T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
        T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
        T::Params: TypedParams<T::SortField, T::FilterField> + Default,
        T::Attrs: for<'b> From<(T, &'b T::Params)>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {
        fn patch(req: &'a mut Request) -> IronResult<Response> {
            match req.get::<bodyparser::Struct<T::Resource>>() {
                Ok(Some(patch)) => {
                    let result = <T as FromPatch<T>>::patch(id(req), patch, Default::default());
                    RequestResult(result, Status::Ok).try_into()
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
