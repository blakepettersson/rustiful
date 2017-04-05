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
use self::iron::mime::Mime;
use queryspec::ToJson;
use errors::RequestError;
use params::JsonApiResource;
use serde::Serialize;
use service::JsonPatch;
use serde::de::Deserialize;
use params::TypedParams;
use errors::RepositoryError;
use errors::QueryStringParseError;
use std::str::FromStr;
use service::JsonGet;
use request::FromPatch;
use request::FromGet;
use request::FromPost;
use service::JsonPost;
use service::JsonDelete;
use service::JsonIndex;
use iron::router::Router;
use request::FromIndex;
use request::FromDelete;

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

fn id<'a>(req: &'a Request) -> &'a str {
    let router = req.extensions
        .get::<router::Router>()
        .expect("Expected to get a Router from the request extensions.");
    router.find("id").expect("No id param found in method that expects one!")
}