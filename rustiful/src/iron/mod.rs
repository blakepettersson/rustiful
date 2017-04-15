mod handlers;

extern crate iron;
extern crate router;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use self::handlers::*;
use self::iron::mime::Mime;
use self::iron::prelude::*;
use self::iron::status;
use error::JsonApiErrorArray;
use errors::QueryStringParseError;
use errors::RequestError;
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
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;

#[derive(Debug)]
struct RequestResult<T, E>(Result<T, RequestError<E>>, Status)
    where T: Serialize,
          E: Send + Error;

impl<T, E> TryFrom<RequestResult<T, E>> for Response
    where T: Serialize,
          E: Send + Error + 'static
{
    type Error = IronError;

    fn try_from(request: RequestResult<T, E>) -> IronResult<Response> {
        let result = request.0;
        let content_type: Mime = "application/vnd.api+json".parse().unwrap();

        match result {
            Ok(json) => {
                match serde_json::to_string(&json) {
                    Ok(serialized) => {
                        let status = request.1;
                        match status {
                            Status::Created | Status::NoContent => {
                                Ok(Response::with((content_type, status)))
                            }
                            _ => Ok(Response::with((content_type, status, serialized))),
                        }
                    }
                    Err(e) => Err(IronError::new(e, Status::InternalServerError)),
                }
            }
            Err(err) => {
                let status = err.status();
                let json = JsonApiErrorArray::new(&err, status);

                match serde_json::to_string(&json) {
                    Ok(serialized) => Ok(Response::with((content_type, status, serialized))),
                    Err(e) => Err(IronError::new(e, Status::InternalServerError)),
                }
            }
        }
    }
}

pub fn id<'a>(req: &'a Request) -> &'a str {
    let router = req.extensions
        .get::<router::Router>()
        .expect("Expected to get a Router from the request extensions.");
    router.find("id").expect("No id param found in method that expects one!")
}

pub trait DeleteRouter {
    fn jsonapi_delete<'a, T>(&mut self)
        where T: JsonDelete + JsonApiResource + ToJson + for<'b> DeleteHandler<'b, T>,
              T::Error: Send + 'static,
              Status: for<'b> From<&'b T::Error>,
              T::JsonApiIdType: FromStr,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static;
}

impl DeleteRouter for Router {
    fn jsonapi_delete<'a, T>(&mut self)
        where T: JsonDelete + JsonApiResource + ToJson + for<'b> DeleteHandler<'b, T>,
              T::Error: Send + 'static,
              Status: for<'b> From<&'b T::Error>,
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
    fn jsonapi_get<'a, T>(&mut self)
        where T: JsonGet + JsonApiResource + ToJson + for<'b> GetHandler<'b, T>,
              T::Error: Send + 'static,
              T::JsonApiIdType: FromStr,
              Status: for<'b> From<&'b T::Error>,
              T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params),
                                         Error = QueryStringParseError>,
              T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params),
                                         Error = QueryStringParseError>,
              T::Params: TypedParams<T::SortField, T::FilterField> + Default,
              T::Attrs: for<'b> From<(T, &'b T::Params)>,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static;
}

impl GetRouter for Router {
    fn jsonapi_get<'a, T>(&mut self)
        where T: JsonGet + JsonApiResource + ToJson + for<'b> GetHandler<'b, T>,
              T::Error: Send + 'static,
              T::JsonApiIdType: FromStr,
              Status: for<'b> From<&'b T::Error>,
              T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params),
                                         Error = QueryStringParseError>,
              T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params),
                                         Error = QueryStringParseError>,
              T::Params: TypedParams<T::SortField, T::FilterField> + Default,
              T::Attrs: for<'b> From<(T, &'b T::Params)>,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {

        self.get(format!("/{}/:id", T::resource_name()),
                 move |r: &mut Request| T::get(r),
                 format!("get_{}", T::resource_name()));
    }
}

pub trait IndexRouter {
    fn jsonapi_index<'a, T>(&mut self)
        where T: JsonIndex + JsonApiResource + ToJson + for<'b> IndexHandler<'b, T>,
              T::Error: Send + 'static,
              T::JsonApiIdType: FromStr,
              Status: for<'b> From<&'b T::Error>,
              T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params),
                                         Error = QueryStringParseError>,
              T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params),
                                         Error = QueryStringParseError>,
              T::Params: TypedParams<T::SortField, T::FilterField> + Default,
              T::Attrs: for<'b> From<(T, &'b T::Params)>,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static;
}

impl IndexRouter for Router {
    fn jsonapi_index<'a, T>(&mut self)
        where T: JsonIndex + JsonApiResource + ToJson + for<'b> IndexHandler<'b, T>,
              T::Error: Send + 'static,
              T::JsonApiIdType: FromStr,
              Status: for<'b> From<&'b T::Error>,
              T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params),
                                         Error = QueryStringParseError>,
              T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params),
                                         Error = QueryStringParseError>,
              T::Params: TypedParams<T::SortField, T::FilterField> + Default,
              T::Attrs: for<'b> From<(T, &'b T::Params)>,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {

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
        Status: for<'b> From<&'b T::Error>,
        T::Resource: Serialize + Deserialize + Clone + 'static + for<'b> From<(T, &'b T::Params)>,
        T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params),
                                    Error = QueryStringParseError>,
        T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params),
                                    Error = QueryStringParseError>,
        T::Attrs: for<'b> From<(T, &'b T::Params)>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static;
}

impl PostRouter for Router {
    fn jsonapi_post<'a, T>(&mut self) where
        T: JsonPost + JsonApiResource + ToJson + for<'b> PostHandler<'b, T>,
        T::Error : Send + 'static,
        T::JsonApiIdType: FromStr,
        Status: for<'b> From<&'b T::Error>,
        T::Resource: Serialize + Deserialize + Clone + 'static + for<'b> From<(T, &'b T::Params)>,
        T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params),
                                    Error = QueryStringParseError>,
        T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params),
                                    Error = QueryStringParseError>,
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
        Status: for<'b> From<&'b T::Error>,
        T::Resource: Serialize + Deserialize + Clone + 'static + for<'b> From<(T, &'b T::Params)>,
        T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params),
                                    Error = QueryStringParseError>,
        T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params),
                                    Error = QueryStringParseError>,
        T::Attrs: for<'b> From<(T, &'b T::Params)>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static;
}

impl PatchRouter for Router {
    fn jsonapi_patch<'a, T>(&mut self) where
        T: JsonPatch + JsonApiResource + ToJson + for<'b> PatchHandler<'b, T>,
        T::Error : Send + 'static,
        T::JsonApiIdType: FromStr,
        Status: for<'b> From<&'b T::Error>,
        T::Resource: Serialize + Deserialize + Clone + 'static + for<'b> From<(T, &'b T::Params)>,
        T::Params: for<'b> TryFrom<(&'b str, Vec<&'b str>, T::Params),
                                    Error = QueryStringParseError>,
        T::Params: for<'b> TryFrom<(&'b str, SortOrder, T::Params),
                                    Error = QueryStringParseError>,
        T::Attrs: for<'b> From<(T, &'b T::Params)>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static {

        self.patch(format!("/{}/:id", T::resource_name()),
                   move |r: &mut Request| T::patch(r),
                   format!("update_{}", T::resource_name()));
    }
}


#[cfg(test)]
mod tests {
    extern crate iron_test;

    use self::iron_test::{request, response};
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
        let req: RequestResult<Test, ParseError> = RequestResult(Ok(test), Status::Ok);
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
        let req: RequestResult<Test, ParseError> = RequestResult(Ok(test), Status::NoContent);
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
        let req: RequestResult<Test, ParseError> = RequestResult(Ok(test), Status::NoContent);
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
        let req: RequestResult<Test, RequestError<ParseError>> =
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
