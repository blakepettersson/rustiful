extern crate iron;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use self::iron::prelude::*;
use super::super::RequestResult;
use errors::QueryStringParseError;
use iron::id;
use params::TypedParams;
use request::FromGet;
use service::JsonGet;
use sort_order::SortOrder;
use status::Status;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;
use try_from::TryInto;

autoimpl! {
    pub trait GetHandler<'a, T> where
        T: JsonGet + ToJson + FromGet<'a, T>,
        T::Error: 'static,
        Status: for<'b> From<&'b T::Error>,
        T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
        T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
        T::Params: TypedParams<T::SortField, T::FilterField> + Default,
        T::Attrs: for<'b> From<(T, &'b T::Params)>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {
        fn get(req: &'a mut Request) -> IronResult<Response> {
            let query = req.url.query().unwrap_or("");
            let result = T::get(id(req), query, Default::default());
            RequestResult(result, Status::Ok).try_into()
        }
    }
}
