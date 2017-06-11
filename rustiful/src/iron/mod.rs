mod handlers;
pub mod from_request;

extern crate iron;
extern crate router;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;
extern crate persistent;

use self::handlers::*;
use self::iron::mime::Mime;
use self::iron::prelude::*;
use self::iron::status;
use self::persistent::Read;
use FromRequest;
use error::JsonApiErrorArray;
use errors::FromRequestError;
use errors::QueryStringParseError;
use errors::RequestError;
use iron::handlers::BodyParserError;
use iron::router::Router;
use serde::Serialize;
use serde::de::Deserialize;
use service::*;
use service::JsonPatch;
use params::SortOrder;
use status::Status;
use std::error::Error;
use std::fmt::Debug;
use std::str::FromStr;
use try_from::TryFrom;
use errors::IdParseError;

fn json_api_type() -> Mime {
    "application/vnd.api+json".parse().unwrap()
}

#[derive(Debug)]
struct RequestResult<T, E>(Result<T, RequestError<E>>, Status) where T: Serialize, E: Send + Error;

impl<T, E> TryFrom<RequestResult<T, E>> for Response where T: Serialize, E: Send + Error + 'static {
    type Error = IronError;

    fn try_from(request: RequestResult<T, E>) -> IronResult<Response> {
        let result = request.0;

        match result {
            Ok(json) => {
                match serde_json::to_string(&json) {
                    Ok(serialized) => {
                        let status = request.1;
                        match status {
                            Status::NoContent => Ok(Response::with((json_api_type(), status))),
                            _ => Ok(Response::with((json_api_type(), status, serialized))),
                        }
                    }
                    Err(e) => Err(IronError::new(e, Status::InternalServerError)),
                }
            }
            Err(err) => err.into(),
        }
    }
}

impl<E> From<RequestError<E>> for IronResult<Response> where E: Send + Error {
    fn from(err: RequestError<E>) -> IronResult<Response> {
        let status = err.status();
        let json = JsonApiErrorArray::new(&err, status);

        match serde_json::to_string(&json) {
            Ok(serialized) => Ok(Response::with((json_api_type(), status, serialized))),
            Err(e) => Err(IronError::new(e, Status::InternalServerError)),
        }
    }
}

impl<T> From<FromRequestError<T>> for IronResult<Response>
    where T: Error + Send
{
    fn from(err: FromRequestError<T>) -> IronResult<Response> {
        let status = Status::InternalServerError;
        let json = JsonApiErrorArray::new(&err, status);

        match serde_json::to_string(&json) {
            Ok(serialized) => Ok(Response::with((json_api_type(), status, serialized))),
            Err(e) => Err(IronError::new(e, status)),
        }
    }
}

impl From<BodyParserError> for IronResult<Response> {
    fn from(err: BodyParserError) -> IronResult<Response> {
        let status = Status::BadRequest;
        let json = JsonApiErrorArray::new(&err, status);

        match serde_json::to_string(&json) {
            Ok(serialized) => Ok(Response::with((json_api_type(), status, serialized))),
            Err(e) => Err(IronError::new(e, status)),
        }
    }
}

impl <E> From<IdParseError<E>> for IronResult<Response> where E: Error {
    fn from(err: IdParseError<E>) -> IronResult<Response> {
        let status = Status::BadRequest;
        let json = JsonApiErrorArray::new(&err, status);

        match serde_json::to_string(&json) {
            Ok(serialized) => Ok(Response::with((json_api_type(), status, serialized))),
            Err(e) => Err(IronError::new(e, status)),
        }
    }
}

fn id<'a>(req: &'a Request) -> &'a str {
    let router = req.extensions
        .get::<Router>()
        .expect("Expected to get a Router from the request extensions.");
    router
        .find("id")
        .expect("No id param found in method that expects one!")
}


/// Constructs a builder for configuring routes for resources implementing any of the `JsonGet`,
/// `JsonPost`, `JsonIndex`, `JsonPatch` or `JsonDelete` traits.
///
/// In order for a resource to be routable we need to configure the routes of the resource. This
/// is what `JsonApiRouterBuilder` does. This builder can create routes for any type implementing
/// any of the `JsonGet`, `JsonPost`, `JsonIndex`, `JsonPatch` or `JsonDelete` traits. We also need
/// to ensure that there are corresponding `From` implementations for all of the distinct error
/// types that are implemented on the above resource traits for `rustiful::status::Status`.
///
/// By default a resource will have a pluralized, lower-cased, dasherized name of its type name (aka
/// kebab-case). If a resource is named `MyResource`, it will have the resource name `my-resources`.
/// This can be overridden by using the Serde `rename` attribute on the resource type.
///
/// # Example
///
/// Given a resource with a valid `JsonIndex` impl, such as the one below:
///
/// ```rust
/// # extern crate iron;
/// # extern crate rustiful;
/// #
/// # #[macro_use]
/// # extern crate rustiful_derive;
/// #
/// # #[macro_use]
/// # extern crate serde_derive;
/// #
/// # use std::default::Default;
/// # use rustiful::JsonApiData;
/// # use rustiful::JsonIndex;
/// # use rustiful::IntoJson;
/// #
/// #[derive(Debug, Default, PartialEq, Eq, Clone, JsonApi)]
/// struct MyResource {
///     id: String,
///     foo: bool,
///     bar: String
/// }
///
/// # struct MyCtx {
/// # }
/// #
/// # impl rustiful::iron::from_request::FromRequest for MyCtx {
/// #     type Error = MyError;
/// #
/// #     fn from_request(req: &iron::request::Request) -> Result<Self, Self::Error> {
/// #         Ok(MyCtx {})
/// #     }
/// # }
/// #
/// # #[derive(Debug)]
/// # struct MyError {
/// # }
/// #
/// # impl std::error::Error for MyError {
/// #    fn description(&self) -> &str {
/// #        "No error here!"
/// #    }
/// #
/// #    fn cause(&self) -> Option<&std::error::Error> {
/// #        None
/// #    }
/// # }
/// #
/// # impl std::fmt::Display for MyError {
/// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
/// #        write!(f, "No error here!")
/// #    }
/// # }
/// #
/// impl JsonIndex for MyResource {
/// #    type Context = MyCtx;
///     type Error = MyError;
/// #
/// #     fn find_all(params: &rustiful::JsonApiParams<Self::FilterField, Self::SortField>,
/// #                 ctx: Self::Context)
/// #            -> Result<Vec<rustiful::JsonApiData<Self>>, Self::Error> {
/// #          Ok(vec![MyResource::default().into_json(params)])
/// #      }
/// }
/// #
/// # fn main() {
/// # }
/// ```
///
/// We also need to ensure that we are able to convert `JsonIndex::Error` to an Iron HTTP status.
///
/// ```
/// # extern crate rustiful;
/// #
/// # struct MyError {
/// # }
/// #
/// impl<'a> From<&'a MyError> for rustiful::status::Status {
///     fn from(error: &'a MyError) -> Self {
///         rustiful::status::ImATeapot
///     }
/// }
/// ```
///
/// Construct a `GET` route for this resource by calling `jsonapi_index`.
///
/// ```rust
/// # extern crate iron;
/// # extern crate rustiful;
/// #
/// # #[macro_use]
/// # extern crate rustiful_derive;
/// #
/// # #[macro_use]
/// # extern crate serde_derive;
/// #
/// # use std::default::Default;
/// # use rustiful::JsonApiData;
/// # use rustiful::JsonIndex;
/// # use rustiful::IntoJson;
/// # use rustiful::iron::JsonApiRouterBuilder;
/// #
/// # #[derive(Debug, Default, PartialEq, Eq, Clone, JsonApi)]
/// # struct MyResource {
/// #    id: String,
/// #    foo: bool,
/// #    bar: String
/// # }
/// #
/// # struct MyCtx {
/// # }
/// #
/// # impl rustiful::iron::from_request::FromRequest for MyCtx {
/// #     type Error = MyError;
/// #
/// #     fn from_request(req: &iron::request::Request) -> Result<Self, Self::Error> {
/// #         Ok(MyCtx {})
/// #     }
/// # }
/// #
/// # #[derive(Debug)]
/// # struct MyError {
/// # }
/// #
/// # impl std::error::Error for MyError {
/// #    fn description(&self) -> &str {
/// #        "No error here!"
/// #    }
/// #
/// #    fn cause(&self) -> Option<&std::error::Error> {
/// #        None
/// #    }
/// # }
/// #
/// # impl std::fmt::Display for MyError {
/// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
/// #        write!(f, "No error here!")
/// #    }
/// # }
/// #
/// # impl JsonIndex for MyResource {
/// #    type Context = MyCtx;
/// #    type Error = MyError;
/// #
/// #     fn find_all(params: &rustiful::JsonApiParams<Self::FilterField, Self::SortField>,
/// #                 ctx: Self::Context)
/// #            -> Result<Vec<rustiful::JsonApiData<Self>>, Self::Error> {
/// #          Ok(vec![MyResource::default().into_json(params)])
/// #      }
/// # }
/// #
/// # impl<'a> From<&'a MyError> for rustiful::status::Status {
/// #   fn from(error: &'a MyError) -> Self {
/// #       rustiful::status::ImATeapot
/// #   }
/// # }
/// #
/// # fn main() {
/// let mut router = JsonApiRouterBuilder::default();
/// router.jsonapi_index::<MyResource>();
/// # }
/// ```
///
/// This resource will then have the route `GET /my-resources`.
///
/// We can then build the `Router` and pass it into the Iron server constructor.
///
/// ```rust,no_run
/// extern crate iron;
/// # extern crate rustiful;
///
/// # use rustiful::iron::JsonApiRouterBuilder;
/// use iron::Iron;
///
/// fn main() {
/// # let router = JsonApiRouterBuilder::default();
///     // Start the server.
///     Iron::new(router.build()).http("localhost:3000").unwrap();
/// }
/// ```
///
/// If you want to change the resource name, we can modify the resource by using the Serde `rename`
/// attribute.
///
/// ```
/// # #[macro_use]
/// # extern crate rustiful_derive;
/// #
/// #[macro_use]
/// extern crate serde_derive;
///
/// #[derive(Debug, Default, PartialEq, Eq, Clone, JsonApi, Serialize, Deserialize)]
/// #[serde(rename = "crazy-resource-name")]
/// struct MyResource {
///     id: String,
///     foo: bool,
///     bar: String
/// }
/// #
/// # fn main() {
/// # }
/// ```
///
/// This resource will then have the route `GET /crazy-resource-name`.
#[allow(missing_debug_implementations)] // The underlying Router doesn't implement Debug...
pub struct JsonApiRouterBuilder {
    router: Router,
    max_body_length: usize,
}

/// This `Default` implementation sets up an Iron `Router` and sets the default bodyparser size to
/// 10MB.
impl Default for JsonApiRouterBuilder {
    fn default() -> Self {
        Self::new(Router::new(), 10 * 1024 * 1024)
    }
}

impl JsonApiRouterBuilder {
    /// Constructs a new `JsonApiRouterBuilder`.
    ///
    /// # Example
    ///
    /// ```
    /// extern crate router;
    /// extern crate rustiful;
    ///
    /// use router::Router;
    /// use rustiful::iron::JsonApiRouterBuilder;
    ///
    /// fn main() {
    ///       // Creates a new JsonApiRouterBuilder with an iron router and a 1MB maximum body
    ///       // length.
    ///       let builder = JsonApiRouterBuilder::new(Router::new(), 1 * 1024 * 1024);
    /// }
    /// ```
    pub fn new(router: Router, max_body_length: usize) -> Self {
        JsonApiRouterBuilder {
            router: router,
            max_body_length: max_body_length,
        }
    }

    /// Sets the max body length for any incoming JSON document. This is specified in bytes.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate rustiful;
    /// # use rustiful::iron::JsonApiRouterBuilder;
    /// #
    /// # fn main() {
    ///       let mut builder = JsonApiRouterBuilder::default();
    ///       // Sets the maximum allowed body length to 1MB.
    ///       builder.set_max_body_length(1 * 1024 * 1024);
    /// # }
    /// ```
    pub fn set_max_body_length(&mut self, max_body_length: usize) {
        self.max_body_length = max_body_length;
    }

    /// Setup a route for a struct that implements `JsonIndex` and `JsonApiResource`
    ///
    /// # Example
    ///
    /// Given a resource with a valid `JsonIndex` impl, such as the one below:
    ///
    /// ```rust
    /// # extern crate iron;
    /// # extern crate rustiful;
    /// #
    /// # #[macro_use]
    /// # extern crate rustiful_derive;
    /// #
    /// # #[macro_use]
    /// # extern crate serde_derive;
    /// #
    /// # use std::default::Default;
    /// # use rustiful::JsonApiData;
    /// # use rustiful::JsonIndex;
    /// # use rustiful::IntoJson;
    /// #
    /// #[derive(Debug, Default, PartialEq, Eq, Clone, JsonApi)]
    /// struct MyResource {
    ///     id: String,
    ///     foo: bool,
    ///     bar: String
    /// }
    ///
    /// # struct MyCtx {
    /// # }
    /// #
    /// # impl rustiful::iron::from_request::FromRequest for MyCtx {
    /// #     type Error = MyError;
    /// #
    /// #     fn from_request(req: &iron::request::Request) -> Result<Self, Self::Error> {
    /// #         Ok(MyCtx {})
    /// #     }
    /// # }
    /// #
    /// # #[derive(Debug)]
    /// # struct MyError {
    /// # }
    /// #
    /// # impl std::error::Error for MyError {
    /// #    fn description(&self) -> &str {
    /// #        "No error here!"
    /// #    }
    /// #
    /// #    fn cause(&self) -> Option<&std::error::Error> {
    /// #        None
    /// #    }
    /// # }
    /// #
    /// # impl std::fmt::Display for MyError {
    /// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    /// #        write!(f, "No error here!")
    /// #    }
    /// # }
    /// #
    /// impl JsonIndex for MyResource {
    /// #    type Context = MyCtx;
    /// #    type Error = MyError;
    /// #
    /// #     fn find_all(params: &rustiful::JsonApiParams<Self::FilterField, Self::SortField>,
    /// #                 ctx: Self::Context)
    /// #            -> Result<Vec<rustiful::JsonApiData<Self>>, Self::Error> {
    /// #          Ok(vec![MyResource::default().into_json(params)])
    /// #      }
    /// }
    /// #
    /// # fn main() {
    /// # }
    /// ```
    ///
    /// Construct a `GET` route for this resource by calling `jsonapi_index`.
    ///
    /// ```rust
    /// # extern crate iron;
    /// # extern crate rustiful;
    /// #
    /// # #[macro_use]
    /// # extern crate rustiful_derive;
    /// #
    /// # #[macro_use]
    /// # extern crate serde_derive;
    /// #
    /// # use std::default::Default;
    /// # use rustiful::JsonApiData;
    /// # use rustiful::JsonIndex;
    /// # use rustiful::IntoJson;
    /// # use rustiful::iron::JsonApiRouterBuilder;
    /// #
    /// # #[derive(Debug, Default, PartialEq, Eq, Clone, JsonApi)]
    /// # struct MyResource {
    /// #    id: String,
    /// #    foo: bool,
    /// #    bar: String
    /// # }
    /// #
    /// # struct MyCtx {
    /// # }
    /// #
    /// # impl rustiful::iron::from_request::FromRequest for MyCtx {
    /// #     type Error = MyError;
    /// #
    /// #     fn from_request(req: &iron::request::Request) -> Result<Self, Self::Error> {
    /// #         Ok(MyCtx {})
    /// #     }
    /// # }
    /// #
    /// # #[derive(Debug)]
    /// # struct MyError {
    /// # }
    /// #
    /// # impl std::error::Error for MyError {
    /// #    fn description(&self) -> &str {
    /// #        "No error here!"
    /// #    }
    /// #
    /// #    fn cause(&self) -> Option<&std::error::Error> {
    /// #        None
    /// #    }
    /// # }
    /// #
    /// # impl std::fmt::Display for MyError {
    /// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    /// #        write!(f, "No error here!")
    /// #    }
    /// # }
    /// #
    /// # impl JsonIndex for MyResource {
    /// #    type Context = MyCtx;
    /// #    type Error = MyError;
    /// #
    /// #     fn find_all(params: &rustiful::JsonApiParams<Self::FilterField, Self::SortField>,
    /// #                 ctx: Self::Context)
    /// #            -> Result<Vec<rustiful::JsonApiData<Self>>, Self::Error> {
    /// #          Ok(vec![MyResource::default().into_json(params)])
    /// #      }
    /// # }
    /// #
    /// # impl<'a> From<&'a MyError> for rustiful::status::Status {
    /// #   fn from(error: &'a MyError) -> Self {
    /// #       rustiful::status::ImATeapot
    /// #   }
    /// # }
    /// #
    /// # fn main() {
    /// let mut router = JsonApiRouterBuilder::default();
    /// router.jsonapi_index::<MyResource>();
    /// # }
    /// ```
    ///
    /// This resource will then have the route `GET /my-resources`.
    pub fn jsonapi_index<'a, T>(&mut self)
        where Status: for<'b> From<&'b T::Error>,
              T: JsonIndex + for<'b> IndexHandler<'b, T>,
              T::Error: 'static,
              T::SortField: for<'b> TryFrom<(&'b str, SortOrder), Error = QueryStringParseError>,
              T::FilterField: for<'b> TryFrom<(&'b str, Vec<&'b str>),
                                              Error = QueryStringParseError>,
              <T::JsonApiIdType as FromStr>::Err: Error + 'static
    {

        self.router
            .get(format!("/{}", T::resource_name()),
                 move |r: &mut Request| T::respond(r),
                 format!("index_{}", T::resource_name()));
    }

    /// Setup a route for a struct that implements `JsonGet` and `JsonApiResource`.
    ///
    /// # Example
    ///
    /// Given a resource with a valid `JsonGet` impl, such as the one below:
    ///
    /// ```rust
    /// # extern crate iron;
    /// # extern crate rustiful;
    /// #
    /// # #[macro_use]
    /// # extern crate rustiful_derive;
    /// #
    /// # #[macro_use]
    /// # extern crate serde_derive;
    /// #
    /// # use std::default::Default;
    /// # use rustiful::JsonApiData;
    /// # use rustiful::JsonGet;
    /// # use rustiful::IntoJson;
    /// #
    /// #[derive(Debug, Default, PartialEq, Eq, Clone, JsonApi)]
    /// struct MyResource {
    ///     id: String,
    ///     foo: bool,
    ///     bar: String
    /// }
    ///
    /// # struct MyCtx {
    /// # }
    /// #
    /// # impl rustiful::iron::from_request::FromRequest for MyCtx {
    /// #     type Error = MyError;
    /// #
    /// #     fn from_request(req: &iron::request::Request) -> Result<Self, Self::Error> {
    /// #         Ok(MyCtx {})
    /// #     }
    /// # }
    /// #
    /// # #[derive(Debug)]
    /// # struct MyError {
    /// # }
    /// #
    /// # impl std::error::Error for MyError {
    /// #    fn description(&self) -> &str {
    /// #        "No error here!"
    /// #    }
    /// #
    /// #    fn cause(&self) -> Option<&std::error::Error> {
    /// #        None
    /// #    }
    /// # }
    /// #
    /// # impl std::fmt::Display for MyError {
    /// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    /// #        write!(f, "No error here!")
    /// #    }
    /// # }
    /// #
    /// impl JsonGet for MyResource {
    /// #    type Context = MyCtx;
    /// #    type Error = MyError;
    /// #
    /// #     fn find(id: Self::JsonApiIdType,
    /// #             params: &rustiful::JsonApiParams<Self::FilterField, Self::SortField>,
    /// #             ctx: Self::Context)
    /// #            -> Result<Option<rustiful::JsonApiData<Self>>, Self::Error> {
    /// #          Ok(Some(MyResource::default().into_json(params)))
    /// #      }
    /// }
    /// #
    /// # fn main() {
    /// # }
    /// ```
    ///
    /// Construct a `GET` route for this resource by calling `jsonapi_get`.
    ///
    /// ```rust
    /// # extern crate iron;
    /// # extern crate rustiful;
    /// #
    /// # #[macro_use]
    /// # extern crate rustiful_derive;
    /// #
    /// # #[macro_use]
    /// # extern crate serde_derive;
    /// #
    /// # use std::default::Default;
    /// # use rustiful::JsonApiData;
    /// # use rustiful::JsonGet;
    /// # use rustiful::IntoJson;
    /// # use rustiful::iron::JsonApiRouterBuilder;
    /// #
    /// # #[derive(Debug, Default, PartialEq, Eq, Clone, JsonApi)]
    /// # struct MyResource {
    /// #    id: String,
    /// #    foo: bool,
    /// #    bar: String
    /// # }
    /// #
    /// # struct MyCtx {
    /// # }
    /// #
    /// # impl rustiful::iron::from_request::FromRequest for MyCtx {
    /// #     type Error = MyError;
    /// #
    /// #     fn from_request(req: &iron::request::Request) -> Result<Self, Self::Error> {
    /// #         Ok(MyCtx {})
    /// #     }
    /// # }
    /// #
    /// # #[derive(Debug)]
    /// # struct MyError {
    /// # }
    /// #
    /// # impl std::error::Error for MyError {
    /// #    fn description(&self) -> &str {
    /// #        "No error here!"
    /// #    }
    /// #
    /// #    fn cause(&self) -> Option<&std::error::Error> {
    /// #        None
    /// #    }
    /// # }
    /// #
    /// # impl std::fmt::Display for MyError {
    /// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    /// #        write!(f, "No error here!")
    /// #    }
    /// # }
    /// #
    /// # impl JsonGet for MyResource {
    /// #    type Context = MyCtx;
    /// #    type Error = MyError;
    /// #
    /// #     fn find(id: Self::JsonApiIdType,
    /// #             params: &rustiful::JsonApiParams<Self::FilterField, Self::SortField>,
    /// #             ctx: Self::Context)
    /// #            -> Result<Option<rustiful::JsonApiData<Self>>, Self::Error> {
    /// #          Ok(Some(MyResource::default().into_json(params)))
    /// #      }
    /// # }
    /// #
    /// # impl<'a> From<&'a MyError> for rustiful::status::Status {
    /// #   fn from(error: &'a MyError) -> Self {
    /// #       rustiful::status::ImATeapot
    /// #   }
    /// # }
    /// #
    /// # fn main() {
    /// let mut router = JsonApiRouterBuilder::default();
    /// router.jsonapi_get::<MyResource>();
    /// # }
    /// ```
    ///
    /// This resource will then have the route `GET /my-resources/{id}`.
    pub fn jsonapi_get<'a, T>(&mut self)
        where Status: for<'b> From<&'b T::Error>,
              T: JsonGet + for<'b> GetHandler<'b, T>,
              T::Error: 'static,
              T::SortField: for<'b> TryFrom<(&'b str, SortOrder), Error = QueryStringParseError>,
              T::FilterField: for<'b> TryFrom<(&'b str, Vec<&'b str>),
                                              Error = QueryStringParseError>,
              <T::JsonApiIdType as FromStr>::Err: Error + 'static
    {

        self.router
            .get(format!("/{}/:id", T::resource_name()),
                 move |r: &mut Request| T::respond(r),
                 format!("get_{}", T::resource_name()));
    }

    /// Setup a route for a struct that implements `JsonDelete` and `JsonApiResource`.
    ///
    /// # Example
    ///
    /// Given a resource with a valid `JsonDelete` impl, such as the one below:
    ///
    /// ```rust
    /// # extern crate iron;
    /// # extern crate rustiful;
    /// #
    /// # #[macro_use]
    /// # extern crate rustiful_derive;
    /// #
    /// # #[macro_use]
    /// # extern crate serde_derive;
    /// #
    /// # use rustiful::JsonApiData;
    /// # use rustiful::JsonDelete;
    /// # use rustiful::IntoJson;
    /// #
    /// #[derive(Debug, Default, PartialEq, Eq, Clone, JsonApi)]
    /// struct MyResource {
    ///     id: String,
    ///     foo: bool,
    ///     bar: String
    /// }
    ///
    /// # struct MyCtx {
    /// # }
    /// #
    /// # impl rustiful::iron::from_request::FromRequest for MyCtx {
    /// #     type Error = MyError;
    /// #
    /// #     fn from_request(req: &iron::request::Request) -> Result<Self, Self::Error> {
    /// #         Ok(MyCtx {})
    /// #     }
    /// # }
    /// #
    /// # #[derive(Debug)]
    /// # struct MyError {
    /// # }
    /// #
    /// # impl std::error::Error for MyError {
    /// #    fn description(&self) -> &str {
    /// #        "No error here!"
    /// #    }
    /// #
    /// #    fn cause(&self) -> Option<&std::error::Error> {
    /// #        None
    /// #    }
    /// # }
    /// #
    /// # impl std::fmt::Display for MyError {
    /// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    /// #        write!(f, "No error here!")
    /// #    }
    /// # }
    /// #
    /// impl JsonDelete for MyResource {
    /// #    type Context = MyCtx;
    /// #    type Error = MyError;
    /// #
    /// #    fn delete(id: Self::JsonApiIdType, ctx: Self::Context) -> Result<(), Self::Error> {
    /// #         Ok(())
    /// #    }
    /// }
    /// #
    /// # fn main() {
    /// # }
    /// ```
    ///
    /// Construct a `DELETE` route for this resource by calling `jsonapi_delete`.
    ///
    /// ```rust
    /// # extern crate iron;
    /// # extern crate rustiful;
    /// #
    /// # #[macro_use]
    /// # extern crate rustiful_derive;
    /// #
    /// # #[macro_use]
    /// # extern crate serde_derive;
    /// #
    /// # use rustiful::JsonApiData;
    /// # use rustiful::JsonDelete;
    /// # use rustiful::IntoJson;
    /// # use rustiful::iron::JsonApiRouterBuilder;
    /// #
    /// # #[derive(Debug, Default, PartialEq, Eq, Clone, JsonApi)]
    /// # struct MyResource {
    /// #    id: String,
    /// #    foo: bool,
    /// #    bar: String
    /// # }
    /// #
    /// # struct MyCtx {
    /// # }
    /// #
    /// # impl rustiful::iron::from_request::FromRequest for MyCtx {
    /// #     type Error = MyError;
    /// #
    /// #     fn from_request(req: &iron::request::Request) -> Result<Self, Self::Error> {
    /// #         Ok(MyCtx {})
    /// #     }
    /// # }
    /// #
    /// # #[derive(Debug)]
    /// # struct MyError {
    /// # }
    /// #
    /// # impl std::error::Error for MyError {
    /// #    fn description(&self) -> &str {
    /// #        "No error here!"
    /// #    }
    /// #
    /// #    fn cause(&self) -> Option<&std::error::Error> {
    /// #        None
    /// #    }
    /// # }
    /// #
    /// # impl std::fmt::Display for MyError {
    /// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    /// #        write!(f, "No error here!")
    /// #    }
    /// # }
    /// #
    /// # impl JsonDelete for MyResource {
    /// #    type Context = MyCtx;
    /// #    type Error = MyError;
    /// #
    /// #    fn delete(id: Self::JsonApiIdType, ctx: Self::Context) -> Result<(), Self::Error> {
    /// #         Ok(())
    /// #    }
    /// # }
    /// #
    /// # impl<'a> From<&'a MyError> for rustiful::status::Status {
    /// #   fn from(error: &'a MyError) -> Self {
    /// #       rustiful::status::ImATeapot
    /// #   }
    /// # }
    /// #
    /// # fn main() {
    /// let mut router = JsonApiRouterBuilder::default();
    /// router.jsonapi_delete::<MyResource>();
    /// # }
    /// ```
    ///
    /// This resource will then have the route `DELETE /my-resources/{id}`.
    pub fn jsonapi_delete<'a, T>(&mut self)
        where Status: for<'b> From<&'b T::Error>,
              T: JsonDelete + for<'b> DeleteHandler<'b, T>,
              T::Error: 'static,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {

        self.router
            .delete(format!("/{}/:id", T::resource_name()),
                    move |r: &mut Request| T::respond(r),
                    format!("delete_{}", T::resource_name()));
    }


    /// Setup a `POST` route for a type that implements `JsonPost` and `JsonApiResource`.
    ///
    /// # Example
    ///
    /// Given a resource with a valid `JsonPost` impl, such as the one below:
    ///
    /// ```rust
    /// # extern crate iron;
    /// # extern crate rustiful;
    /// #
    /// # #[macro_use]
    /// # extern crate rustiful_derive;
    /// #
    /// # #[macro_use]
    /// # extern crate serde_derive;
    /// #
    /// # use rustiful::JsonApiData;
    /// # use rustiful::JsonPost;
    /// # use rustiful::IntoJson;
    /// #
    /// #[derive(Debug, Default, PartialEq, Eq, Clone, JsonApi)]
    /// struct MyResource {
    ///     id: String,
    ///     foo: bool,
    ///     bar: String
    /// }
    ///
    /// # struct MyCtx {
    /// # }
    /// #
    /// # impl rustiful::iron::from_request::FromRequest for MyCtx {
    /// #     type Error = MyError;
    /// #
    /// #     fn from_request(req: &iron::request::Request) -> Result<Self, Self::Error> {
    /// #         Ok(MyCtx {})
    /// #     }
    /// # }
    /// #
    /// # #[derive(Debug)]
    /// # struct MyError {
    /// # }
    /// #
    /// # impl std::error::Error for MyError {
    /// #    fn description(&self) -> &str {
    /// #        "No error here!"
    /// #    }
    /// #
    /// #    fn cause(&self) -> Option<&std::error::Error> {
    /// #        None
    /// #    }
    /// # }
    /// #
    /// # impl std::fmt::Display for MyError {
    /// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    /// #        write!(f, "No error here!")
    /// #    }
    /// # }
    /// #
    /// impl JsonPost for MyResource {
    /// #    type Context = MyCtx;
    /// #    type Error = MyError;
    /// #
    /// #    fn create(json: rustiful::JsonApiData<Self>,
    /// #         params: &rustiful::JsonApiParams<Self::FilterField, Self::SortField>,
    /// #          ctx: Self::Context)
    /// #          -> Result<rustiful::JsonApiData<Self>, Self::Error> {
    /// #         Ok(MyResource::default().into_json(params))
    /// #    }
    /// }
    /// #
    /// # fn main() {
    /// # }
    /// ```
    ///
    /// Construct a `POST` route for this resource by calling `jsonapi_post`.
    ///
    /// ```rust
    /// # extern crate iron;
    /// # extern crate rustiful;
    /// #
    /// # #[macro_use]
    /// # extern crate rustiful_derive;
    /// #
    /// # #[macro_use]
    /// # extern crate serde_derive;
    /// #
    /// # use rustiful::JsonApiData;
    /// # use rustiful::JsonPost;
    /// # use rustiful::IntoJson;
    /// # use rustiful::iron::JsonApiRouterBuilder;
    /// #
    /// # #[derive(Debug, Default, PartialEq, Eq, Clone, JsonApi)]
    /// # struct MyResource {
    /// #    id: String,
    /// #    foo: bool,
    /// #    bar: String
    /// # }
    /// #
    /// # struct MyCtx {
    /// # }
    /// #
    /// # impl rustiful::iron::from_request::FromRequest for MyCtx {
    /// #     type Error = MyError;
    /// #
    /// #     fn from_request(req: &iron::request::Request) -> Result<Self, Self::Error> {
    /// #         Ok(MyCtx {})
    /// #     }
    /// # }
    /// #
    /// # #[derive(Debug)]
    /// # struct MyError {
    /// # }
    /// #
    /// # impl std::error::Error for MyError {
    /// #    fn description(&self) -> &str {
    /// #        "No error here!"
    /// #    }
    /// #
    /// #    fn cause(&self) -> Option<&std::error::Error> {
    /// #        None
    /// #    }
    /// # }
    /// #
    /// # impl std::fmt::Display for MyError {
    /// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    /// #        write!(f, "No error here!")
    /// #    }
    /// # }
    /// #
    /// # impl JsonPost for MyResource {
    /// #    type Context = MyCtx;
    /// #    type Error = MyError;
    /// #
    /// #    fn create(json: rustiful::JsonApiData<Self>,
    /// #         params: &rustiful::JsonApiParams<Self::FilterField, Self::SortField>,
    /// #          ctx: Self::Context)
    /// #          -> Result<rustiful::JsonApiData<Self>, Self::Error> {
    /// #         let resource = MyResource {
    /// #             id: "some_id".to_string(),
    /// #             foo: true,
    /// #             bar: "abc".to_string()
    /// #         };
    /// #
    /// #         Ok(resource.into_json(params))
    /// #    }
    /// # }
    /// #
    /// # impl<'a> From<&'a MyError> for rustiful::status::Status {
    /// #   fn from(error: &'a MyError) -> Self {
    /// #       rustiful::status::ImATeapot
    /// #   }
    /// # }
    /// #
    /// # fn main() {
    /// let mut router = JsonApiRouterBuilder::default();
    /// router.jsonapi_post::<MyResource>();
    /// # }
    /// ```
    /// This resource will then have the route `POST /my-resources`.
    pub fn jsonapi_post<'a, T>(&mut self)
        where Status: for<'b> From<&'b T::Error>,
              T: 'static + JsonPost + for<'b> PostHandler<'b, T>,
              T::Error: 'static,
              T::Attrs: 'static + for<'b> Deserialize<'b>,
              T::SortField: for<'b> TryFrom<(&'b str, SortOrder), Error = QueryStringParseError>,
              T::FilterField: for<'b> TryFrom<(&'b str, Vec<&'b str>),
                                              Error = QueryStringParseError>,
              <T::Context as FromRequest>::Error: 'static,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {

        self.router
            .post(format!("/{}", T::resource_name()),
                  move |r: &mut Request| T::respond(r),
                  format!("create_{}", T::resource_name()));
    }

    /// Configures a route for a type that implements `JsonPatch` and `JsonApiResource`.
    ///
    /// # Example
    ///
    /// Given a resource with a valid `JsonPatch` impl, such as the one below:
    ///
    /// ```rust
    /// # extern crate iron;
    /// # extern crate rustiful;
    /// #
    /// # #[macro_use]
    /// # extern crate rustiful_derive;
    /// #
    /// # #[macro_use]
    /// # extern crate serde_derive;
    /// #
    /// # use rustiful::JsonApiData;
    /// # use rustiful::JsonPatch;
    /// # use rustiful::IntoJson;
    /// #
    /// #[derive(Debug, Default, PartialEq, Eq, Clone, JsonApi)]
    /// struct MyResource {
    ///     id: String,
    ///     foo: bool,
    ///     bar: String
    /// }
    ///
    /// # struct MyCtx {
    /// # }
    /// #
    /// # impl rustiful::iron::from_request::FromRequest for MyCtx {
    /// #     type Error = MyError;
    /// #
    /// #     fn from_request(req: &iron::request::Request) -> Result<Self, Self::Error> {
    /// #         Ok(MyCtx {})
    /// #     }
    /// # }
    /// #
    /// # #[derive(Debug)]
    /// # struct MyError {
    /// # }
    /// #
    /// # impl std::error::Error for MyError {
    /// #    fn description(&self) -> &str {
    /// #        "No error here!"
    /// #    }
    /// #
    /// #    fn cause(&self) -> Option<&std::error::Error> {
    /// #        None
    /// #    }
    /// # }
    /// #
    /// # impl std::fmt::Display for MyError {
    /// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    /// #        write!(f, "No error here!")
    /// #    }
    /// # }
    /// #
    /// impl JsonPatch for MyResource {
    /// #    type Context = MyCtx;
    /// #    type Error = MyError;
    /// #
    /// #    fn update(id: Self::JsonApiIdType,
    /// #              json: rustiful::JsonApiData<Self>,
    /// #              params: &rustiful::JsonApiParams<Self::FilterField, Self::SortField>,
    /// #              ctx: Self::Context)
    /// #              -> Result<rustiful::JsonApiData<Self>, Self::Error> {
    /// #         let resource = MyResource {
    /// #             id: "some_id".to_string(),
    /// #             foo: true,
    /// #             bar: "abc".to_string()
    /// #         };
    /// #
    /// #         Ok(resource.into_json(params))
    /// #    }
    /// }
    /// #
    /// # fn main() {
    /// # }
    /// ```
    ///
    /// Construct a `PATCH` route for this resource by calling `jsonapi_patch`.
    ///
    /// ```rust
    /// # extern crate iron;
    /// # extern crate rustiful;
    /// #
    /// # #[macro_use]
    /// # extern crate rustiful_derive;
    /// #
    /// # #[macro_use]
    /// # extern crate serde_derive;
    /// #
    /// # use rustiful::JsonApiData;
    /// # use rustiful::JsonPatch;
    /// # use rustiful::IntoJson;
    /// # use rustiful::iron::JsonApiRouterBuilder;
    /// #
    /// # #[derive(Debug, Default, PartialEq, Eq, Clone, JsonApi)]
    /// # struct MyResource {
    /// #    id: String,
    /// #    foo: bool,
    /// #    bar: String
    /// # }
    /// #
    /// # struct MyCtx {
    /// # }
    /// #
    /// # impl rustiful::iron::from_request::FromRequest for MyCtx {
    /// #     type Error = MyError;
    /// #
    /// #     fn from_request(req: &iron::request::Request) -> Result<Self, Self::Error> {
    /// #         Ok(MyCtx {})
    /// #     }
    /// # }
    /// #
    /// # #[derive(Debug)]
    /// # struct MyError {
    /// # }
    /// #
    /// # impl std::error::Error for MyError {
    /// #    fn description(&self) -> &str {
    /// #        "No error here!"
    /// #    }
    /// #
    /// #    fn cause(&self) -> Option<&std::error::Error> {
    /// #        None
    /// #    }
    /// # }
    /// #
    /// # impl std::fmt::Display for MyError {
    /// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    /// #        write!(f, "No error here!")
    /// #    }
    /// # }
    /// #
    /// # impl JsonPatch for MyResource {
    /// #    type Context = MyCtx;
    /// #    type Error = MyError;
    /// #
    /// #    fn update(id: Self::JsonApiIdType,
    /// #              json: rustiful::JsonApiData<Self>,
    /// #              params: &rustiful::JsonApiParams<Self::FilterField, Self::SortField>,
    /// #              ctx: Self::Context)
    /// #              -> Result<rustiful::JsonApiData<Self>, Self::Error> {
    /// #         let resource = MyResource {
    /// #             id: "some_id".to_string(),
    /// #             foo: true,
    /// #             bar: "abc".to_string()
    /// #         };
    /// #
    /// #         Ok(resource.into_json(params))
    /// #    }
    /// # }
    /// #
    /// # impl<'a> From<&'a MyError> for rustiful::status::Status {
    /// #   fn from(error: &'a MyError) -> Self {
    /// #       rustiful::status::ImATeapot
    /// #   }
    /// # }
    /// #
    /// # fn main() {
    /// let mut router = JsonApiRouterBuilder::default();
    /// router.jsonapi_patch::<MyResource>();
    /// # }
    /// ```
    ///
    /// This resource will then have the route `PATCH /my-resources/{id}`.
    pub fn jsonapi_patch<'a, T>(&mut self)
        where Status: for<'b> From<&'b T::Error>,
              T: 'static + JsonPatch + for<'b> PatchHandler<'b, T>,
              T::Error: 'static,
              T::Attrs: 'static + for<'b> Deserialize<'b>,
              T::SortField: for<'b> TryFrom<(&'b str, SortOrder), Error = QueryStringParseError>,
              T::FilterField: for<'b> TryFrom<(&'b str, Vec<&'b str>),
                                              Error = QueryStringParseError>,
              <T::JsonApiIdType as FromStr>::Err: Error + 'static
    {

        self.router
            .patch(format!("/{}/:id", T::resource_name()),
                   move |r: &mut Request| T::respond(r),
                   format!("update_{}", T::resource_name()));
    }

    /// Constructs an iron `Chain` with the routes that were previously specified in `jsonapi_get`,
    /// `jsonapi_post` et cetera. This also sets up the body parser, which is a prerequisite for
    /// being able to parse JSON documents when doing a `POST` or `PATCH`. The result of this method
    /// can then be used in the Iron server constructor.
    ///
    /// ```rust,no_run
    /// extern crate iron;
    /// # extern crate rustiful;
    ///
    /// # use rustiful::iron::JsonApiRouterBuilder;
    /// # use iron::Iron;
    ///
    /// # fn main() {
    /// let router = JsonApiRouterBuilder::default();
    /// // Start the server.
    /// Iron::new(router.build()).http("localhost:3000").unwrap();
    /// # }
    /// ```
    pub fn build(self) -> Chain {
        let mut chain = iron::Chain::new(self.router);
        chain.link_before(Read::<bodyparser::MaxBodyLength>::one(self.max_body_length));
        chain
    }
}

#[cfg(test)]
mod tests {
    extern crate iron_test;

    use self::iron_test::response;
    use super::*;
    use super::iron::headers::ContentType;
    use error::JsonApiError;
    use std::string::ParseError;
    use try_from::TryInto;

    #[derive(Serialize, Deserialize)]
    struct Test {
        foo: String,
    }

    #[test]
    fn test_200_ok_response() {
        let test = Test { foo: "bar".to_string() };
        let req: RequestResult<Test, ParseError, String> = RequestResult(Ok(test), Status::Ok);
        let resp: IronResult<Response> = req.try_into();
        let result = resp.expect("Invalid response!");
        let headers = result.headers.clone();
        let content_type = headers
            .get::<ContentType>()
            .expect("no content type found!");
        assert_eq!("application/vnd.api+json", content_type.to_string());
        let json = response::extract_body_to_string(result);
        assert_eq!("{\"foo\":\"bar\"}", json);
    }

    #[test]
    fn test_201_created() {
        let test = Test { foo: "bar".to_string() };
        let req: RequestResult<Test, ParseError> = RequestResult(Ok(test),
                                                                         Status::NoContent);
        let resp: IronResult<Response> = req.try_into();
        let result = resp.expect("Invalid response!");
        let headers = result.headers.clone();
        let content_type = headers
            .get::<ContentType>()
            .expect("no content type found!");
        assert_eq!("application/vnd.api+json", content_type.to_string());
        let json = response::extract_body_to_string(result);
        assert_eq!("", json);
    }

    #[test]
    fn test_204_no_content() {
        let test = Test { foo: "bar".to_string() };
        let req: RequestResult<Test, ParseError> = RequestResult(Ok(test),
                                                                         Status::NoContent);
        let resp: IronResult<Response> = req.try_into();
        let result = resp.expect("Invalid response!");
        let headers = result.headers.clone();
        let content_type = headers
            .get::<ContentType>()
            .expect("no content type found!");
        assert_eq!("application/vnd.api+json", content_type.to_string());
        let json = response::extract_body_to_string(result);
        assert_eq!("", json);
    }

    #[test]
    fn test_error_json() {
        let req: RequestResult<Test, RequestError<ParseError>> =
            RequestResult(Err(RequestError::NoBody), Status::NoContent);
        let resp: IronResult<Response> = req.try_into();
        let result = resp.expect("Invalid response!");
        let headers = result.headers.clone();
        let content_type = headers
            .get::<ContentType>()
            .expect("no content type found!");
        assert_eq!("application/vnd.api+json", content_type.to_string());
        let json = response::extract_body_to_string(result);
        let expected = JsonApiErrorArray {
            errors: vec![JsonApiError {
                             detail: "No body".to_string(),
                             status: "400".to_string(),
                             title: "No body".to_string(),
                         }],
        };
        let record: JsonApiErrorArray = serde_json::from_str(&json).unwrap();
        assert_eq!(expected, record);
    }
}
