mod handlers;

mod from_request;
pub use self::from_request::*;

mod router_builder;
pub use self::router_builder::*;

extern crate iron;
extern crate router;
extern crate bodyparser;
extern crate serde;
extern crate serde_json;

use self::iron::mime::Mime;
use self::iron::prelude::*;
use self::status::Status;
use container::JsonApiContainer;
use error::JsonApiErrorArray;
use iron::router::Router;
use resource::JsonApiResource;
use serde::Serialize;
use service::Handler;
use std::error::Error;

impl<T: JsonApiResource> Handler for T {
    type Status = Status;
}

/// Status Codes
pub mod status {
    extern crate hyper;

    pub use self::hyper::status::StatusClass;
    pub use self::hyper::status::StatusCode as Status;
    pub use self::hyper::status::StatusCode::*;
}

fn json_api_type() -> Mime {
    "application/vnd.api+json".parse().unwrap()
}

#[derive(Debug)]
struct JsonErrorResponse<E: Error>(E, Status);

impl<E: Error> From<JsonErrorResponse<E>> for IronResult<Response> {
    fn from(err: JsonErrorResponse<E>) -> IronResult<Response> {
        match serde_json::to_string(&JsonApiErrorArray::new(&err.0, err.1.to_u16())) {
            Ok(serialized) => Ok(Response::with((json_api_type(), err.1, serialized))),
            Err(e) => Err(IronError::new(e, Status::InternalServerError))
        }
    }
}

#[derive(Debug)]
struct JsonOkResponse<T: Serialize>(T);

impl<T: Serialize> From<JsonOkResponse<T>> for IronResult<Response> {
    fn from(data: JsonOkResponse<T>) -> IronResult<Response> {
        let json = JsonApiContainer { data: data.0 };
        match serde_json::to_string(&json) {
            Ok(serialized) => Ok(Response::with((json_api_type(), Status::Ok, serialized))),
            Err(e) => Err(IronError::new(e, Status::InternalServerError))
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
    use errors::IdParseError;
    use errors::QueryStringParseError;
    use serde::de::DeserializeOwned;
    use std::collections::HashMap;

    // Use `QueryStringParseError` to be wrapped in `RepositoryError`, for convenience
    impl<'a> From<&'a QueryStringParseError> for Status {
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

        let json = assert_response(JsonOkResponse(data).into(), Status::Ok);
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

        let json = assert_response(JsonOkResponse(vec![data]).into(), Status::Ok);
        let expected = r#"{"data":[{"id":"foo","type":"foos","attributes":{"foo":"bar"}}]}"#;
        assert_json_success::<JsonApiContainer<Vec<Data>>>(expected, &json);
    }

    #[test]
    fn from_query_parse_error_conversion() {
        let error = QueryStringParseError::UnImplementedError;
        let json = assert_response(
            JsonErrorResponse(error, Status::BadRequest).into(),
            Status::BadRequest
        );
        let expected = r#"{"errors":[{"title":"Unimplemented","status":"400","detail":"Query string parse error: Unimplemented!"}]}"#;
        assert_json_error(expected, &json);
    }

    #[test]
    fn from_body_parser_error_conversion() {
        let serde_fail = serde_json::from_str::<JsonApiErrorArray>("");
        let error = BodyError {
            detail: "test".to_string(),
            cause: BodyErrorCause::JsonError(serde_fail.expect_err("unexpected ok!"))
        };
        let json = assert_response(
            JsonErrorResponse(error, Status::BadRequest).into(),
            Status::BadRequest
        );
        let expected = r#"{"errors":[{"title":"test","status":"400","detail":"test"}]}"#;
        assert_json_error(expected, &json);
    }

    #[test]
    fn from_id_parse_error_conversion() {
        let parse_fail: Result<u8, _> = "not a string".parse();
        let error = IdParseError(parse_fail.expect_err("unexpected ok!"));
        let json = assert_response(
            JsonErrorResponse(error, Status::BadRequest).into(),
            Status::BadRequest
        );

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
