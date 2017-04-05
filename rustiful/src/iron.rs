extern crate iron;
extern crate router;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use try_from::TryFrom;
use sort_order::SortOrder;
use std::error::Error;
use self::iron::prelude::*;
use self::iron::status;
use to_json::ToJson;
use params::JsonApiResource;
use serde::Serialize;
use service::JsonPatch;
use serde::de::Deserialize;
use params::TypedParams;
use errors::RepositoryError;
use errors::QueryStringParseError;
use std::str::FromStr;
use iron_handlers::*;
use service::*;
use iron::router::Router;

impl From<RepositoryError> for IronError {
    fn from(err: RepositoryError) -> IronError {
        IronError::new(err, status::InternalServerError)
    }
}

impl From<QueryStringParseError> for IronError {
    fn from(err: QueryStringParseError) -> IronError {
        IronError::new(err, status::InternalServerError)
    }
}

pub trait DeleteRouter {
    fn jsonapi_delete<'a, T>(&mut self)
        where T: JsonDelete + JsonApiResource + ToJson + for<'b> DeleteHandler<'b, T>,
              T::Error: Send + 'static,
              T::JsonApiIdType: FromStr,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static;
}

impl DeleteRouter for Router {
    fn jsonapi_delete<'a, T>(&mut self)
        where T: JsonDelete + JsonApiResource + ToJson + for<'b> DeleteHandler<'b, T>,
              T::Error: Send + 'static,
              T::JsonApiIdType: FromStr,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {

        self.delete(format!("/{}/:id", T::resource_name()),
                    move |r: &mut Request| {
                        <T as DeleteHandler<T>>::delete(r)
                    },
                    format!("delete_{}", T::resource_name()));
    }
}

pub trait GetRouter {
    fn jsonapi_get<'a, T>(&mut self) where
        T: JsonGet + JsonApiResource + ToJson + for<'b> GetHandler<'b, T>,
        T::Error : Send + 'static,
        T::JsonApiIdType: FromStr,
        T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params), Error = QueryStringParseError>,
        T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params), Error = QueryStringParseError>,
        T::Params: TypedParams<T::SortField, T::FilterField> + Default,
        T::Attrs: for<'b> From<(T, &'b T::Params)>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static;
}

impl GetRouter for Router {
    fn jsonapi_get<'a, T>(&mut self) where
        T: JsonGet + JsonApiResource + ToJson + for<'b> GetHandler<'b, T>,
        T::Error : Send + 'static,
        T::JsonApiIdType: FromStr,
        T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params), Error = QueryStringParseError>,
        T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params), Error = QueryStringParseError>,
        T::Params: TypedParams<T::SortField, T::FilterField> + Default,
        T::Attrs: for<'b> From<(T, &'b T::Params)>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static {

        self.get(format!("/{}/:id", T::resource_name()),
                 move |r: &mut Request| T::get(r),
                 format!("get_{}", T::resource_name()));
    }
}

pub trait IndexRouter {
    fn jsonapi_index<'a, T>(&mut self) where
        T: JsonIndex + JsonApiResource + ToJson + for<'b> IndexHandler<'b, T>,
        T::Error : Send + 'static,
        T::JsonApiIdType: FromStr,
        T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params), Error = QueryStringParseError>,
        T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params), Error = QueryStringParseError>,
        T::Params: TypedParams<T::SortField, T::FilterField> + Default,
        T::Attrs: for<'b> From<(T, &'b T::Params)>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static;
}

impl IndexRouter for Router {
    fn jsonapi_index<'a, T>(&mut self) where
        T: JsonIndex + JsonApiResource + ToJson + for<'b> IndexHandler<'b, T>,
        T::Error : Send + 'static,
        T::JsonApiIdType: FromStr,
        T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params), Error = QueryStringParseError>,
        T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params), Error = QueryStringParseError>,
        T::Params: TypedParams<T::SortField, T::FilterField> + Default,
        T::Attrs: for<'b> From<(T, &'b T::Params)>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static {

        self.get(format!("/{}", T::resource_name()),
                 move |r: &mut Request| T::get(r),
                 format!("index_{}", T::resource_name()));
    }
}

pub trait PostRouter {
    fn jsonapi_post<'a, T>(&mut self) where
        T: JsonPost + JsonApiResource + ToJson + for<'b> PostHandler<'b, T>,
        T::Error : Send + 'static,
        T::JsonApiIdType: FromStr,
        T::Resource: Serialize + Deserialize + Clone + 'static + for<'b> From<(T, &'b T::Params)>,
        T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params), Error = QueryStringParseError>,
        T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params), Error = QueryStringParseError>,
        T::Attrs: for<'b> From<(T, &'b T::Params)>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static;
}

impl PostRouter for Router {
    fn jsonapi_post<'a, T>(&mut self) where
        T: JsonPost + JsonApiResource + ToJson + for<'b> PostHandler<'b, T>,
        T::Error : Send + 'static,
        T::JsonApiIdType: FromStr,
        T::Resource: Serialize + Deserialize + Clone + 'static + for<'b> From<(T, &'b T::Params)>,
        T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params), Error = QueryStringParseError>,
        T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params), Error = QueryStringParseError>,
        T::Attrs: for<'b> From<(T, &'b T::Params)>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static {

        self.post(format!("/{}", T::resource_name()),
                 move |r: &mut Request| T::post(r),
                 format!("create_{}", T::resource_name()));
    }
}

pub trait PatchRouter {
    fn jsonapi_patch<'a, T>(&mut self) where
        T: JsonPatch + JsonApiResource + ToJson + for<'b> PatchHandler<'b, T>,
        T::Error : Send + 'static,
        T::JsonApiIdType: FromStr,
        T::Resource: Serialize + Deserialize + Clone + 'static + for<'b> From<(T, &'b T::Params)>,
        T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params), Error = QueryStringParseError>,
        T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params), Error = QueryStringParseError>,
        T::Attrs: for<'b> From<(T, &'b T::Params)>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static;
}

impl PatchRouter for Router {
    fn jsonapi_patch<'a, T>(&mut self) where
        T: JsonPatch + JsonApiResource + ToJson + for<'b> PatchHandler<'b, T>,
        T::Error : Send + 'static,
        T::JsonApiIdType: FromStr,
        T::Resource: Serialize + Deserialize + Clone + 'static + for<'b> From<(T, &'b T::Params)>,
        T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params), Error = QueryStringParseError>,
        T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params), Error = QueryStringParseError>,
        T::Attrs: for<'b> From<(T, &'b T::Params)>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static {

        self.patch(format!("/{}/:id", T::resource_name()),
                  move |r: &mut Request| T::patch(r),
                  format!("update_{}", T::resource_name()));
    }
}
