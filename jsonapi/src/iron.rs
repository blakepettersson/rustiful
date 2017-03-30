extern crate iron;
extern crate router;
extern crate serde;
extern crate serde_json;

use std::error::Error;
use params::Params;
use request::FromRequest;
use self::iron::prelude::*;
use self::iron::status;
use self::iron::mime::Mime;
use queryspec::ToJson;
use service::JsonApiService;
use query_string::QueryString;
use params::JsonApiResource;
use try_from::TryFrom;
use sort_order::SortOrder;
use params::TypedParams;
use queryspec::QueryStringParseError;
use errors::RepositoryError;
use std::str::FromStr;

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

pub trait IronHandlers<'a, T> where
    T: Default,
    T: FromRequest<'a, Self::Resource, Self::Params, Self::SortField, Self::FilterField>,
    T: JsonApiService<T = Self::Resource>,
    <T as JsonApiService>::Error: 'static,
    Self::Params: for<'b> TryFrom<(&'b str, SortOrder, Self::Params), Err = QueryStringParseError>,
    Self::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, Self::Params), Err = QueryStringParseError>,
    <Self::Resource as ToJson>::Attrs: for<'b> From<(Self::Resource, &'b Self::Params)>,
    <Self::Resource as JsonApiResource>::JsonApiIdType: FromStr,
    <<Self::Resource as JsonApiResource>::JsonApiIdType as FromStr>::Err: Send,
    <<Self::Resource as JsonApiResource>::JsonApiIdType as FromStr>::Err: Error,
    <<Self::Resource as JsonApiResource>::JsonApiIdType as FromStr>::Err: 'static {

    type Params: Params + Default + TypedParams<SortField = Self::SortField, FilterField = Self::FilterField>;
    type Resource: ToJson + for<'b> QueryString<'b, Params = Self::Params, SortField = Self::SortField, FilterField = Self::FilterField> + JsonApiResource<Params = Self::Params>;
    type SortField;
    type FilterField;

    type GetHandler: GetHandler<'a, T, Self::Resource, Self::Params, Self::SortField, Self::FilterField> + iron::Handler + 'static;
    type IndexHandler: IndexHandler<'a, T, Self::Resource, Self::Params, Self::SortField, Self::FilterField> + iron::Handler + 'static;
}


pub trait GetHandler<'a, T, R, P, S, F> where
    T : Default,
    T : FromRequest<'a, R, P, S, F>,
    T: JsonApiService<T = R>,
    <T as JsonApiService>::Error: 'static,
    P: Params,
    P: Default,
    P: TypedParams<SortField = S, FilterField = F>,
    P: for<'b> TryFrom<(&'b str, SortOrder, P), Err = QueryStringParseError>,
    P: for<'b> TryFrom<(&'b str, Vec<&'b str>, P), Err = QueryStringParseError>,
    R: ToJson,
    R: for<'b> QueryString<'b, Params = P, SortField = S, FilterField = F>,
    R: JsonApiResource<Params = P>,
    <R as ToJson>::Attrs: for<'b> From<(R, &'b P)>,
    <R as JsonApiResource>::JsonApiIdType: FromStr,
    <<R as JsonApiResource>::JsonApiIdType as FromStr>::Err: Send,
    <<R as JsonApiResource>::JsonApiIdType as FromStr>::Err: Error,
    <<R as JsonApiResource>::JsonApiIdType as FromStr>::Err: 'static
{
    fn get(req: &'a mut Request) -> IronResult<Response> {
        let content_type:Mime = "application/vnd.api+json".parse().unwrap();

        let router = req.extensions
            .get::<router::Router>()
            .expect("Expected to get a Router from the request extensions.");

        let query = req.url.query().unwrap_or("");
        let repository:T = Default::default();
        let id = router.find("id").unwrap();
        match repository.get(id, query) {
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

pub trait IndexHandler<'a, T, R, P, S, F> where
    T : Default,
    T : FromRequest<'a, R, P, S, F>,
    T: JsonApiService<T = R>,
    <T as JsonApiService>::Error: 'static,
    P: Params,
    P: Default,
    P: TypedParams<SortField = S, FilterField = F>,
    P: for<'b> TryFrom<(&'b str, SortOrder, P), Err = QueryStringParseError>,
    P: for<'b> TryFrom<(&'b str, Vec<&'b str>, P), Err = QueryStringParseError>,
    R: ToJson,
    R: for<'b> QueryString<'b, Params = P, SortField = S, FilterField = F>,
    R: JsonApiResource<Params = P>,
    <R as ToJson>::Attrs: for<'b> From<(R, &'b P)>,
    <R as JsonApiResource>::JsonApiIdType: FromStr,
    <<R as JsonApiResource>::JsonApiIdType as FromStr>::Err: Send,
    <<R as JsonApiResource>::JsonApiIdType as FromStr>::Err: Error,
    <<R as JsonApiResource>::JsonApiIdType as FromStr>::Err: 'static
{
    fn index(req: &'a mut Request) -> IronResult<Response> {
        let content_type:Mime = "application/vnd.api+json".parse().unwrap();

        let query = req.url.query().unwrap_or("");
        let repository:T = Default::default();
        match repository.index(query) {
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