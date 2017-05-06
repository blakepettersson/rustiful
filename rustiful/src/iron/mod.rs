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
use params::JsonApiResource;
use params::TypedParams;
use serde::Serialize;
use serde::de::Deserialize;
use service::*;
use service::JsonPatch;
use sort_order::SortOrder;
use status::Status;
use std::error::Error;
use std::fmt::Debug;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;

fn json_api_type() -> Mime {
    "application/vnd.api+json".parse().unwrap()
}

#[derive(Debug)]
struct RequestResult<T, E, I>(Result<T, RequestError<E, I>>, Status)
    where T: Serialize,
          E: Send + Error,
          I: FromStr + Debug,
          <I as FromStr>::Err: Error;

impl<T, E, I> TryFrom<RequestResult<T, E, I>> for Response
    where T: Serialize,
          E: Send + Error + 'static,
          I: FromStr + Debug,
          <I as FromStr>::Err: Error
{
    type Error = IronError;

    fn try_from(request: RequestResult<T, E, I>) -> IronResult<Response> {
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

impl<E, I> From<RequestError<E, I>> for IronResult<Response>
    where E: Send + Error,
          I: FromStr + Debug,
          <I as FromStr>::Err: Error
{
    fn from(err: RequestError<E, I>) -> IronResult<Response> {
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

pub fn id<'a>(req: &'a Request) -> &'a str {
    let router = req.extensions
        .get::<Router>()
        .expect("Expected to get a Router from the request extensions.");
    router.find("id").expect("No id param found in method that expects one!")
}


/// This is used to construct an Iron `Chain`, and also sets up a bodyparser with a default max body
/// length.
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
    fn new(router: Router, max_body_length: usize) -> Self {
        JsonApiRouterBuilder {
            router: router,
            max_body_length: max_body_length,
        }
    }

    /// Sets the max body length for any incoming JSON document. This is specified in bytes.
    pub fn set_max_body_length(&mut self, max_body_length: usize) {
        self.max_body_length = max_body_length;
    }

    /// Setup a route for a struct that implements `JsonIndex` and `JsonApiResource`
    pub fn jsonapi_index<'a, T>(&mut self)
        where Status: for<'b> From<&'b T::Error>,
              T: JsonIndex + JsonApiResource + ToJson + for<'b> IndexHandler<'b, T>,
              T::Error: Send + 'static,
              T::JsonApiIdType: FromStr,
              T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params),
                  Error = QueryStringParseError>,
              T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params),
                  Error = QueryStringParseError>,
              T::Params: TypedParams<T::SortField, T::FilterField> + Default,
              T::Attrs: for<'b> From<(T, &'b T::Params)>,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static,
              <<T as JsonIndex>::Context as FromRequest>::Error: 'static,
    {

        self.router.get(format!("/{}", T::resource_name()),
                        move |r: &mut Request| T::get(r),
                        format!("index_{}", T::resource_name()));
    }

    /// Setup a route for a struct that implements `JsonGet` and `JsonApiResource`.
    pub fn jsonapi_get<'a, T>(&mut self)
        where Status: for<'b> From<&'b T::Error>,
              T: JsonGet + JsonApiResource + ToJson + for<'b> GetHandler<'b, T>,
              T::Error: Send + 'static,
              T::JsonApiIdType: FromStr,
              T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params),
                  Error = QueryStringParseError>,
              T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params),
                  Error = QueryStringParseError>,
              T::Params: TypedParams<T::SortField, T::FilterField> + Default,
              T::Attrs: for<'b> From<(T, &'b T::Params)>,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static,
              <<T as JsonGet>::Context as FromRequest>::Error: 'static
    {

        self.router.get(format!("/{}/:id", T::resource_name()),
                        move |r: &mut Request| T::get(r),
                        format!("get_{}", T::resource_name()));
    }

    /// Setup a route for a struct that implements `JsonDelete` and `JsonApiResource`.
    pub fn jsonapi_delete<'a, T>(&mut self)
        where Status: for<'b> From<&'b T::Error>,
              T: JsonDelete + JsonApiResource + ToJson + for<'b> DeleteHandler<'b, T>,
              T::Error: Send + 'static,
              T::JsonApiIdType: FromStr,
              <T::Context as FromRequest>::Error: 'static,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {

        self.router.delete(format!("/{}/:id", T::resource_name()),
                           move |r: &mut Request| {
                               <T as DeleteHandler<T>>::delete(r)
                           },
                           format!("delete_{}", T::resource_name()));
    }


    /// Setup a route for a struct that implements `JsonPost` and `JsonApiResource`.
    pub fn jsonapi_post<'a, T>(&mut self)
        where Status: for<'b> From<&'b T::Error>,
              T: JsonPost + JsonApiResource + ToJson + for<'b> PostHandler<'b, T>,
              T::Error: Send + 'static,
              T::JsonApiIdType: FromStr,
              T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params),
                                         Error = QueryStringParseError>,
              T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params),
                                         Error = QueryStringParseError>,
              T::Attrs: for<'b> From<(T, &'b T::Params)> + 'static + for<'b> Deserialize<'b>,
              <T::Context as FromRequest>::Error: 'static,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {

        self.router.post(format!("/{}", T::resource_name()),
                         move |r: &mut Request| T::post(r),
                         format!("create_{}", T::resource_name()));
    }

    /// Setup a route for a struct that implements `JsonPatch` and `JsonApiResource`.
    pub fn jsonapi_patch<'a, T>(&mut self)
        where Status: for<'b> From<&'b T::Error>,
              T: JsonPatch + JsonApiResource + ToJson + for<'b> PatchHandler<'b, T>,
              T::Error: Send + 'static,
              T::JsonApiIdType: FromStr,
              T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params),
                                         Error = QueryStringParseError>,
              T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params),
                                         Error = QueryStringParseError>,
              T::Attrs: for<'b> From<(T, &'b T::Params)> + 'static + for<'b> Deserialize<'b>,
              <T::Context as FromRequest>::Error: 'static,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {

        self.router.patch(format!("/{}/:id", T::resource_name()),
                          move |r: &mut Request| T::patch(r),
                          format!("update_{}", T::resource_name()));
    }

    /// Constructs an iron `Chain` with the routes that were previously specified in `jsonapi_get`,
    /// `jsonapi_post` et cetera. This also sets up the body parser, which is a prerequisite for
    /// being able to parse JSON documents when POSTing or PATCHing a document.
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
        let content_type = headers.get::<ContentType>().expect("no content type found!");
        assert_eq!("application/vnd.api+json", content_type.to_string());
        let json = response::extract_body_to_string(result);
        assert_eq!("{\"foo\":\"bar\"}", json);
    }

    #[test]
    fn test_201_created() {
        let test = Test { foo: "bar".to_string() };
        let req: RequestResult<Test, ParseError, String> = RequestResult(Ok(test),
                                                                         Status::NoContent);
        let resp: IronResult<Response> = req.try_into();
        let result = resp.expect("Invalid response!");
        let headers = result.headers.clone();
        let content_type = headers.get::<ContentType>().expect("no content type found!");
        assert_eq!("application/vnd.api+json", content_type.to_string());
        let json = response::extract_body_to_string(result);
        assert_eq!("", json);
    }

    #[test]
    fn test_204_no_content() {
        let test = Test { foo: "bar".to_string() };
        let req: RequestResult<Test, ParseError, String> = RequestResult(Ok(test),
                                                                         Status::NoContent);
        let resp: IronResult<Response> = req.try_into();
        let result = resp.expect("Invalid response!");
        let headers = result.headers.clone();
        let content_type = headers.get::<ContentType>().expect("no content type found!");
        assert_eq!("application/vnd.api+json", content_type.to_string());
        let json = response::extract_body_to_string(result);
        assert_eq!("", json);
    }

    #[test]
    fn test_error_json() {
        let req: RequestResult<Test, RequestError<ParseError, String>, String> =
            RequestResult(Err(RequestError::NoBody), Status::NoContent);
        let resp: IronResult<Response> = req.try_into();
        let result = resp.expect("Invalid response!");
        let headers = result.headers.clone();
        let content_type = headers.get::<ContentType>().expect("no content type found!");
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
