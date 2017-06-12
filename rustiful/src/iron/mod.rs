mod handlers;
pub mod from_request;

mod router_builder;
pub use self::router_builder::*;

extern crate iron;
extern crate router;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use self::iron::mime::Mime;
use self::iron::prelude::*;
use self::iron::status;
use container::JsonApiContainer;
use error::JsonApiErrorArray;
use errors::FromRequestError;
use errors::IdParseError;
use errors::QueryStringParseError;
use errors::RepositoryError;
use errors::RequestError;
use iron::handlers::BodyParserError;
use iron::router::Router;
use serde::Serialize;
use status::Status;
use std::error::Error;

fn json_api_type() -> Mime {
    "application/vnd.api+json".parse().unwrap()
}

fn into_json_api_response<T>(data: T, status: Status) -> IronResult<Response>
    where T: Serialize
{
    let json = JsonApiContainer { data: data };

    match serde_json::to_string(&json) {
        Ok(serialized) => Ok(Response::with((json_api_type(), status, serialized))),
        Err(e) => Err(IronError::new(e, Status::InternalServerError)),
    }
}

impl From<RequestError> for IronResult<Response> {
    fn from(err: RequestError) -> IronResult<Response> {
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

impl From<QueryStringParseError> for IronResult<Response> {
    fn from(err: QueryStringParseError) -> IronResult<Response> {
        let status = Status::BadRequest;
        let json = JsonApiErrorArray::new(&err, status);

        match serde_json::to_string(&json) {
            Ok(serialized) => Ok(Response::with((json_api_type(), status, serialized))),
            Err(e) => Err(IronError::new(e, status)),
        }
    }
}

impl<E> From<RepositoryError<E>> for IronResult<Response>
    where E: Error + Send
{
    fn from(err: RepositoryError<E>) -> IronResult<Response> {
        let status = err.status;
        let json = JsonApiErrorArray::new(&err, status);

        match serde_json::to_string(&json) {
            Ok(serialized) => Ok(Response::with((json_api_type(), status, serialized))),
            Err(e) => Err(IronError::new(e, status)),
        }
    }
}

impl<E> From<IdParseError<E>> for IronResult<Response>
    where E: Error
{
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

#[cfg(test)]
mod tests {
    extern crate iron_test;
    extern crate bodyparser;

    use self::bodyparser::*;

    use self::iron_test::response;
    use super::*;
    use super::iron::headers::ContentType;
    use std::collections::HashMap;
    use serde::de::DeserializeOwned;

    // Use `QueryStringParseError` to be wrapped in `RepositoryError`, for convenience
    impl <'a> From<&'a QueryStringParseError> for Status {
        fn from(_: &'a QueryStringParseError) -> Self {
            Status::ImATeapot
        }
    }

    // A hacky version of `JsonApiData<T>`, since the attributes field requires a type that
    // implements `ToJson`
    #[derive(Serialize, Deserialize)]
    struct Data {
        pub id: String,
        #[serde(rename = "type")]
        // The type name of the JSONAPI resource, equivalent to the resource name.
        pub lower_case_type: String,
        pub attributes: HashMap<String, String>
    }

    #[test]
    fn into_json_api_response_object() {
        let mut attributes = HashMap::new();
        attributes.insert("foo".to_string(), "bar".to_string());
        let data = Data {
            id: "foo".to_string(),
            lower_case_type: "foos".to_string(),
            attributes: attributes
        };

        let result = into_json_api_response(data, Status::Ok);
        let json = assert_response(result, Status::Ok);
        let expected = r#"{"data":{"id":"foo","type":"foos","attributes":{"foo":"bar"}}}"#;
        assert_json_success::<JsonApiContainer<Data>>(expected, &json);
    }

    #[test]
    fn into_json_api_response_list() {
        let mut attributes = HashMap::new();
        attributes.insert("foo".to_string(), "bar".to_string());
        let data = Data {
            id: "foo".to_string(),
            lower_case_type: "foos".to_string(),
            attributes: attributes
        };

        let result = into_json_api_response(vec![data], Status::Ok);
        let json = assert_response(result, Status::Ok);
        let expected = r#"{"data":[{"id":"foo","type":"foos","attributes":{"foo":"bar"}}]}"#;
        assert_json_success::<JsonApiContainer<Vec<Data>>>(expected, &json);
    }

    #[test]
    fn from_request_error_400_conversion() {
        let error = RequestError::NoBody;
        let json = assert_response(error.into(), Status::BadRequest);
        let expected = r#"{"errors":[{"title":"No body","status":"400","detail":"No body"}]}"#;
        assert_json_error(expected, &json);
    }

    #[test]
    fn from_request_error_404_conversion() {
        let error = RequestError::NotFound;
        let json = assert_response(error.into(), Status::NotFound);
        let expected = r#"{"errors":[{"title":"Not found","status":"404","detail":"Not found"}]}"#;
        assert_json_error(expected, &json);
    }

    #[test]
    fn from_from_request_error_conversion() {
        let error = FromRequestError(RequestError::NotFound);
        let json = assert_response(error.into(), Status::InternalServerError);
        let expected = r#"{"errors":[{"title":"Not found","status":"500","detail":"From request error: Not found"}]}"#;
        assert_json_error(expected, &json);
    }

    #[test]
    fn from_query_parse_error_conversion() {
        let error = QueryStringParseError::UnImplementedError;
        let json = assert_response(error.into(), Status::BadRequest);
        let expected = r#"{"errors":[{"title":"Unimplemented","status":"400","detail":"Query string parse error: Unimplemented!"}]}"#;
        assert_json_error(expected, &json);
    }

    #[test]
    fn from_repository_error_conversion() {
        let error = RepositoryError::new(QueryStringParseError::UnImplementedError);
        let json = assert_response(error.into(), Status::ImATeapot);
        let expected = r#"{"errors":[{"title":"Unimplemented","status":"418","detail":"Error in repository: Unimplemented"}]}"#;
        assert_json_error(expected, &json);
    }

    #[test]
    fn from_body_parser_error_conversion() {
        let serde_fail = serde_json::from_str::<JsonApiErrorArray>("");
        let error = BodyParserError(BodyError {
            detail: "test".to_string(),
            cause: BodyErrorCause::JsonError(serde_fail.expect_err("unexpected ok!"))
        });
        let json = assert_response(error.into(), Status::BadRequest);
        let expected = r#"{"errors":[{"title":"test","status":"400","detail":"Error when parsing json: test"}]}"#;
        assert_json_error(expected, &json);
    }

    #[test]
    fn from_id_parse_error_conversion() {
        let parse_fail: Result<u8, _> = "not a string".parse();
        let error = IdParseError(parse_fail.expect_err("unexpected ok!"));
        let json = assert_response(error.into(), Status::BadRequest);

        let expected = r#"{"errors":[{"title":"invalid digit found in string","status":"400","detail":"Error parsing id: invalid digit found in string"}]}"#;
        assert_json_error(expected, &json);
    }

    fn assert_response(response: IronResult<Response>, status: Status) -> String {
        let result = response.expect("Invalid response!");
        let headers = result.headers.clone();
        let content_type = headers
            .get::<ContentType>()
            .expect("no content type found!");
        assert_eq!(result.status, Some(status));
        assert_eq!("application/vnd.api+json", content_type.to_string());
        response::extract_body_to_string(result)
    }

    fn assert_json_error(expected: &str, result: &str) {
        assert_eq!(expected, result);
        serde_json::from_str::<JsonApiErrorArray>(result).expect("Cannot deserialize json!");

    }

    fn assert_json_success<T: DeserializeOwned>(expected: &str, result: &str) {
        assert_eq!(expected, result);
        serde_json::from_str::<T>(result).expect("Cannot deserialize json!");
    }
}
