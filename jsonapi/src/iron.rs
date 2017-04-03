extern crate iron;
extern crate router;
extern crate serde;
extern crate serde_json;

use std::error::Error;
use self::iron::prelude::*;
use self::iron::status;
use self::iron::mime::Mime;
use queryspec::ToJson;
use query_string::QueryString;
use params::JsonApiResource;
use queryspec::QueryStringParseError;
use errors::RepositoryError;
use std::str::FromStr;
use service::JsonGet;
use request::FromGet;
use service::JsonDelete;
use service::JsonIndex;
use iron::router::Router;
use request::FromIndex;
use request::FromDelete;
use std::marker::PhantomData;

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
    T: JsonGet + ToJson + QueryString<'a> + FromGet<'a, T>,
    T::Error: 'static,
    T::Context: Default,
    T::JsonApiIdType: FromStr,
    T::Attrs: for<'b> From<(T, &'b <T as JsonApiResource>::Params)>,
    <T as JsonApiResource>::Params: From<<T as QueryString<'a>>::Params>,
    <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
{
    fn get(req: &'a mut Request) -> IronResult<Response> {
        let content_type:Mime = "application/vnd.api+json".parse().unwrap();

        let router = req.extensions
            .get::<router::Router>()
            .expect("Expected to get a Router from the request extensions.");

        let query = req.url.query().unwrap_or("");
        let repository:<T as JsonGet>::Context = Default::default();
        let id = router.find("id").unwrap();
        match <T as FromGet<T>>::get(id, query, repository) {
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
    T: JsonGet + ToJson + QueryString<'a> + FromGet<'a, T>,
    T::Error: 'static,
    T::Context: Default,
    T::JsonApiIdType: FromStr,
    T::Attrs: for<'b> From<(T, &'b <T as JsonApiResource>::Params)>,
    <T as JsonApiResource>::Params: From<<T as QueryString<'a>>::Params>,
    <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static {}

pub trait IndexHandler<'a, T> where
    T: JsonIndex + ToJson + QueryString<'a> + FromIndex<'a, T>,
    T::Context : Default,
    T::Error: Send + 'static,
    T::Attrs: for<'b> From<(T, &'b <T as JsonApiResource>::Params)>,
    <T as JsonApiResource>::Params: From<<T as QueryString<'a>>::Params>
{
    fn get(req: &'a mut Request) -> IronResult<Response> {
        let content_type:Mime = "application/vnd.api+json".parse().unwrap();
        let query = req.url.query().unwrap_or("");
        let repository:<T as JsonIndex>::Context = Default::default();
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
    T: JsonIndex + ToJson + QueryString<'a> + FromIndex<'a, T>,
    T::Context : Default,
    T::Error: Send + 'static,
    T::Attrs: for<'b> From<(T, &'b <T as JsonApiResource>::Params)>,
    <T as JsonApiResource>::Params: From<<T as QueryString<'a>>::Params> {}

pub trait DeleteHandler<'a, T> where
    T: JsonDelete + ToJson + QueryString<'a> + FromDelete<'a, T>,
    T::Error : Send + 'static,
    T::Context: Default,
    T::JsonApiIdType: FromStr,
    T::Attrs: for<'b> From<(T, &'b <T as JsonApiResource>::Params)>,
    <T as JsonApiResource>::Params: From<<T as QueryString<'a>>::Params>,
    <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
{
    fn delete(req: &'a mut Request) -> IronResult<Response> {
        let content_type:Mime = "application/vnd.api+json".parse().unwrap();

        let router = req.extensions
            .get::<router::Router>()
            .expect("Expected to get a Router from the request extensions.");

        let repository:<T as JsonDelete>::Context = Default::default();
        let id = router.find("id").unwrap();
        match <T as FromDelete<T>>::delete(id, repository) {
            Ok(_) => {
                Ok(Response::with((content_type, status::NoContent)))
            },
            Err(e) => Err(IronError::new(e, status::InternalServerError))
        }

    }
}

impl <'a, T> DeleteHandler<'a, T> for T where
    T: JsonDelete + ToJson + QueryString<'a> + FromDelete<'a, T>,
    T::Error : Send + 'static,
    T::Context: Default,
    T::JsonApiIdType: FromStr,
    T::Attrs: for<'b> From<(T, &'b <T as JsonApiResource>::Params)>,
    <T as JsonApiResource>::Params: From<<T as QueryString<'a>>::Params>,
    <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {}


/*
pub trait DeleteRouter {
    fn jsonapi_delete<'a, T>(&mut self, _: PhantomData<T>) where
        T: JsonDelete + JsonApiResource + ToJson + for<'b> QueryString<'b> + for<'b> DeleteHandler<'b, T>,
        T::Error : Send + 'static,
        T::JsonApiIdType: FromStr,
        T::Attrs: for<'b> From<(T, &'b <T as JsonApiResource>::Params)>,
        <T as JsonApiResource>::Params: for<'b> From<<T as QueryString<'b>>::Params>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static;
}

impl DeleteRouter for Router {
    fn jsonapi_delete<'a, T>(&mut self, _: PhantomData<T>) where
        T: JsonDelete + JsonApiResource + ToJson + for<'b> QueryString<'b> + for<'b> DeleteHandler<'b, T>,
        T::Error : Send + 'static,
        T::JsonApiIdType: FromStr,
        T::Attrs: for<'b> From<(T, &'b <T as JsonApiResource>::Params)>,
        <T as JsonApiResource>::Params: for<'b> From<<T as QueryString<'b>>::Params>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static {

        self.delete("/foos/:id", move |r: &mut Request| {
            <T as DeleteHandler<T>>::delete(r)
        }, "delete_foo");
    }
}*/