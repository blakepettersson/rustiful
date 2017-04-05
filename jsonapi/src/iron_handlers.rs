extern crate iron;
extern crate router;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use std::error::Error;
use std::str::FromStr;

use self::iron::prelude::*;
use self::iron::status;
use self::iron::mime::Mime;

use serde::Serialize;
use serde::Deserialize;

use errors::*;
use service::*;
use request::*;
use to_json::ToJson;
use try_from::TryFrom;
use params::TypedParams;
use sort_order::SortOrder;
use errors::QueryStringParseError;

pub trait GetHandler<'a, T> where
    T: JsonGet + ToJson +
    FromGet<'a, T>,
    T::Error: 'static,
    T::Context: Default,
    T::JsonApiIdType: FromStr,
    T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
    T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
    T::Params: TypedParams<T::SortField, T::FilterField> + Default,
    T::Attrs: for<'b> From<(T, &'b T::Params)>,
    <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
{
    fn get(req: &'a mut Request) -> IronResult<Response> {
        let content_type:Mime = "application/vnd.api+json".parse().unwrap();
        let query = req.url.query().unwrap_or("");
        let repository:T::Context = Default::default();
        match T::get(id(req), query, repository) {
            Ok(json) => {
                match serde_json::to_string(&json) {
                    Ok(serialized) => Ok(Response::with((content_type, status::Ok, serialized))),
                    Err(e) => Err(IronError::new(e, status::InternalServerError))
                }
            },
            Err(e) => Err(IronError::new(e, status::InternalServerError))
        }
    }
}

impl <'a, T> GetHandler<'a, T> for T where
    T: JsonGet + ToJson + FromGet<'a, T>,
    T::Error: 'static,
    T::Context: Default,
    T::JsonApiIdType: FromStr,
    T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
    T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
    T::Params: TypedParams<T::SortField, T::FilterField> + Default,
    T::Attrs: for<'b> From<(T, &'b T::Params)>,
    <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static {}

pub trait PostHandler<'a, T> where
    T: JsonPost + ToJson + FromPost<'a, T>,
    T::Error: 'static + Send,
    T::Context: Default,
    T::JsonApiIdType: FromStr,
    T::Error: Send,
    T::Resource: Serialize + Deserialize + Clone + 'static + for<'b> From<(T, &'b T::Params)>,
    T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
    T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
    T::Params: TypedParams<T::SortField, T::FilterField> + Default,
    T::Attrs: for<'b> From<(T, &'b T::Params)>,
    <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
{
    fn post(req: &'a mut Request) -> IronResult<Response> {
        let content_type:Mime = "application/vnd.api+json".parse().unwrap();

        match req.get::<bodyparser::Struct<T::Resource>>() {
            Ok(Some(result)) => {
                let repository:T::Context = Default::default();
                match <T as FromPost<T>>::create(result, repository) {
                    Ok(json) => {
                        match serde_json::to_string(&json) {
                            Ok(serialized) => {
                                Ok(Response::with((content_type, status::Ok, serialized)))
                            },
                            Err(e) => Err(IronError::new(e, status::InternalServerError))
                        }
                    },
                    Err(e) => Err(IronError::new(e, status::InternalServerError))
                }
            },
            Ok(None) => Err(IronError::new(RequestError::NoBody, status::InternalServerError)),
            Err(e) => Err(IronError::new(e, status::InternalServerError))
        }
    }
}

impl <'a, T> PostHandler<'a, T> for T where
    T: JsonPost + ToJson + FromPost<'a, T>,
    T::Error: 'static + Send,
    T::Context: Default,
    T::JsonApiIdType: FromStr,
    T::Error: Send,
    T::Resource: Serialize + Deserialize + Clone + 'static + for<'b> From<(T, &'b T::Params)>,
    T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
    T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
    T::Params: TypedParams<T::SortField, T::FilterField> + Default,
    T::Attrs: for<'b> From<(T, &'b T::Params)>,
    <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
{}

pub trait PatchHandler<'a, T> where
    T: JsonPatch + ToJson + FromPatch<'a, T>,
    T::Error: 'static + Send,
    T::Context: Default,
    T::JsonApiIdType: FromStr,
    T::Error: Send,
    T::Resource: Serialize + Deserialize + Clone + 'static + for<'b> From<(T, &'b T::Params)>,
    T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
    T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
    T::Params: TypedParams<T::SortField, T::FilterField> + Default,
    T::Attrs: for<'b> From<(T, &'b T::Params)>,
    <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
{
    fn patch(req: &'a mut Request) -> IronResult<Response> {
        let content_type:Mime = "application/vnd.api+json".parse().unwrap();

        match req.get::<bodyparser::Struct<T::Resource>>() {
            Ok(Some(result)) => {
                let repository: T::Context = Default::default();
                match <T as FromPatch<T>>::patch(id(req), result, repository) {
                    Ok(json) => {
                        match serde_json::to_string(&json) {
                            Ok(serialized) => {
                                Ok(Response::with((content_type, status::Ok, serialized)))
                            },
                            Err(e) => Err(IronError::new(e, status::InternalServerError))
                        }
                    },
                    Err(e) => Err(IronError::new(e, status::InternalServerError))
                }
            },
            Ok(None) => Err(IronError::new(RequestError::NoBody, status::InternalServerError)),
            Err(e) => Err(IronError::new(e, status::InternalServerError))
        }
    }
}

impl <'a, T> PatchHandler<'a, T> for T where
    T: JsonPatch + ToJson + FromPatch<'a, T>,
    T::Error: 'static + Send,
    T::Context: Default,
    T::JsonApiIdType: FromStr,
    T::Error: Send,
    T::Resource: Serialize + Deserialize + Clone + 'static + for<'b> From<(T, &'b T::Params)>,
    T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
    T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
    T::Params: TypedParams<T::SortField, T::FilterField> + Default,
    T::Attrs: for<'b> From<(T, &'b T::Params)>,
    <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
{}

pub trait IndexHandler<'a, T> where
    T: JsonIndex + ToJson +
    FromIndex<'a, T>,
    T::Context : Default,
    T::Error: Send + 'static,
    T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
    T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
    T::Params: TypedParams<T::SortField, T::FilterField> + Default,
    T::Attrs: for<'b> From<(T, &'b T::Params)>
{
    fn get(req: &'a mut Request) -> IronResult<Response> {
        let content_type:Mime = "application/vnd.api+json".parse().unwrap();
        let query = req.url.query().unwrap_or("");
        let repository:T::Context = Default::default();
        match <T as FromIndex<T>>::get(query, repository) {
            Ok(json) => {
                match serde_json::to_string(&json) {
                    Ok(serialized) => Ok(Response::with((content_type, status::Ok, serialized))),
                    Err(e) => Err(IronError::new(e, status::InternalServerError))
                }
            },
            Err(e) => Err(IronError::new(e, status::InternalServerError))
        }

    }
}

impl <'a, T> IndexHandler<'a, T> for T where
    T: JsonIndex + ToJson + FromIndex<'a, T>,
    T::Context : Default,
    T::Error: Send + 'static,
    T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
    T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
    T::Params: TypedParams<T::SortField, T::FilterField> + Default,
    T::Attrs: for<'b> From<(T, &'b T::Params)> {}

pub trait DeleteHandler<'a, T>
    where T: JsonDelete + ToJson + FromDelete<'a, T>,
          T::Error: Send + 'static,
          T::Context: Default,
          T::JsonApiIdType: FromStr,
          <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
{
    fn delete(req: &'a mut Request) -> IronResult<Response> {
        let content_type: Mime = "application/vnd.api+json".parse().unwrap();
        let repository: T::Context = Default::default();
        match <T as FromDelete<T>>::delete(id(req), repository) {
            Ok(_) => Ok(Response::with((content_type, status::NoContent))),
            Err(e) => Err(IronError::new(e, status::InternalServerError)),
        }

    }
}

impl<'a, T> DeleteHandler<'a, T> for T
    where T: JsonDelete + ToJson + FromDelete<'a, T>,
          T::Error: Send + 'static,
          T::Context: Default,
          T::JsonApiIdType: FromStr,
          <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
{
}

fn id<'a>(req: &'a Request) -> &'a str {
    let router = req.extensions
        .get::<router::Router>()
        .expect("Expected to get a Router from the request extensions.");
    router.find("id").expect("No id param found in method that expects one!")
}