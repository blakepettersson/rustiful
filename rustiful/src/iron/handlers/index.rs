extern crate iron;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use self::iron::prelude::*;
use super::super::RequestResult;
use errors::QueryStringParseError;
use params::TypedParams;
use request::FromIndex;
use service::JsonIndex;
use sort_order::SortOrder;
use status::Status;
use to_json::ToJson;
use try_from::TryFrom;
use try_from::TryInto;

autoimpl! {
    pub trait IndexHandler<'a, T> where
        T: JsonIndex + ToJson + FromIndex<'a, T>,
        T::Error: 'static,
        Status: for<'b> From<&'b T::Error>,
        T::Attrs: for<'b> From<(T, &'b T::Params)>,
        T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
        T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
        T::Params: TypedParams<T::SortField, T::FilterField> + Default
    {
        fn get(req: &'a mut Request) -> IronResult<Response> {
            let query = req.url.query().unwrap_or("");
            let result = <T as FromIndex<T>>::get(query, Default::default());
            RequestResult(result, Status::Ok).try_into()
        }
    }
}
