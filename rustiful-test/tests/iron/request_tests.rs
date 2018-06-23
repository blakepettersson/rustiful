extern crate iron;

use self::iron::Chain;
use self::iron::Headers;
use self::iron::headers::ContentType;
use self::iron::mime::Mime;
use self::iron::prelude::*;
use super::iron_test::{request, response};
use resources::mock_resource::*;
use rustiful::*;
use rustiful::iron::*;
use rustiful::iron::status::Status;
use serde_json;
use std::str::FromStr;

impl FromRequest for FooService {
    type Error = TestError;
    fn from_request(request: &Request) -> Result<Self, (Self::Error, Status)> {
        if let Some(_) = request.headers.get_raw("test-fail") {
            return Err((
                TestError("from request fail".to_string()),
                Status::InternalServerError
            ));
        }
        Ok(FooService {})
    }
}

impl From<TestError> for (TestError, Status) {
    fn from(err: TestError) -> Self {
        (err, Status::ImATeapot)
    }
}

fn app_router() -> Chain {
    let mut router = JsonApiRouterBuilder::default();
    router.jsonapi_get::<Foo>();
    router.jsonapi_post::<Foo>();
    router.jsonapi_index::<Foo>();
    router.jsonapi_delete::<Foo>();
    router.jsonapi_patch::<Foo>();
    router.build()
}

#[test]
fn parse_json_api_index_get() {
    let headers = Headers::new();
    let response = request::get("http://localhost:3000/foos", headers, &app_router()).unwrap();
    let headers = response.headers.clone();
    let content_type = headers
        .get::<ContentType>()
        .expect("no content type found!");
    let result = response::extract_body_to_string(response);
    let records: JsonApiContainer<Vec<JsonApiData<Foo>>> = serde_json::from_str(&result).unwrap();
    let params = <Foo as JsonApiResource>::Params::from_str("").expect("failed to unwrap params");

    let test = Foo {
        id: "1".to_string(),
        body: "test".to_string(),
        title: "test".to_string(),
        published: true
    };
    let data: JsonApiData<Foo> = (test, &params).into();
    let expected = JsonApiContainer { data: vec![data] };

    assert_eq!(expected, records);
}

#[test]
fn parse_json_api_index_get_with_fieldset() {
    let headers = Headers::new();
    let response = request::get(
        "http://localhost:3000/foos?fields[foos]=title",
        headers,
        &app_router()
    ).unwrap();
    let headers = response.headers.clone();
    let content_type = headers
        .get::<ContentType>()
        .expect("no content type found!");
    let result = response::extract_body_to_string(response);
    let records: JsonApiContainer<Vec<JsonApiData<Foo>>> = serde_json::from_str(&result).unwrap();
    let data = JsonApiData::new(
        Some("1"),
        <Foo as ToJson>::Attrs::new(Some("test".to_string()), None, None)
    );
    let expected = JsonApiContainer { data: vec![data] };

    assert_eq!(expected, records);
}


#[test]
fn parse_json_api_single_get() {
    let response = request::get(
        "http://localhost:3000/foos/1",
        Headers::new(),
        &app_router()
    );
    let result = response::extract_body_to_string(response.unwrap());
    let record: JsonApiContainer<JsonApiData<Foo>> = serde_json::from_str(&result).unwrap();
    let params = <Foo as JsonApiResource>::Params::from_str("").expect("failed to unwrap params");

    let test = Foo {
        id: "1".to_string(),
        body: "test".to_string(),
        title: "test".to_string(),
        published: true
    };
    let data: JsonApiData<Foo> = (test, &params).into();
    let expected = JsonApiContainer { data: data };

    assert_eq!(expected, record);
}

#[test]
fn parse_json_api_single_get_fail_in_from_request() {
    let mut headers = Headers::new();
    headers.set_raw("test-fail", vec![]);

    let response = request::get("http://localhost:3000/foos/1", headers, &app_router());

    assert_json_api_error(
        response,
        JsonApiError {
            title: "from request fail".to_string(),
            detail: "from request fail".to_string(),
            status: "500".to_string()
        }
    );
}

#[test]
fn parse_json_api_index_get_fail_in_from_request() {
    let mut headers = Headers::new();
    headers.set_raw("test-fail", vec![]);

    let response = request::get("http://localhost:3000/foos", headers, &app_router());

    assert_json_api_error(
        response,
        JsonApiError {
            title: "from request fail".to_string(),
            detail: "from request fail".to_string(),
            status: "500".to_string()
        }
    );
}

#[test]
fn parse_json_api_delete_fail_in_from_request() {
    let mut headers = Headers::new();
    headers.set_raw("test-fail", vec![]);

    let response = request::delete("http://localhost:3000/foos/2", headers, &app_router());

    assert_json_api_error(
        response,
        JsonApiError {
            title: "from request fail".to_string(),
            detail: "from request fail".to_string(),
            status: "500".to_string()
        }
    );
}

#[test]
fn parse_json_api_post_fail_in_from_request() {
    let content_type: Mime = "application/vnd.api+json".parse().unwrap();

    let mut headers = Headers::new();
    headers.set_raw("test-fail", vec![]);
    headers.set::<ContentType>(ContentType(content_type));

    let data = r#"
    {
        "data": {
            "id": "fail",
            "type": "foos",
            "attributes": {
                "title": "test"
            }
        }
    }"#;

    let response = request::post("http://localhost:3000/foos", headers, &data, &app_router());

    assert_json_api_error(
        response,
        JsonApiError {
            title: "from request fail".to_string(),
            detail: "from request fail".to_string(),
            status: "500".to_string()
        }
    );
}

#[test]
fn parse_json_api_patch_fail_in_from_request() {
    let content_type: Mime = "application/vnd.api+json".parse().unwrap();

    let mut headers = Headers::new();
    headers.set_raw("test-fail", vec![]);
    headers.set::<ContentType>(ContentType(content_type));

    let data = r#"
    {
        "data": {
            "id": "fail",
            "type": "foos",
            "attributes": {
                "title": "test"
            }
        }
    }"#;

    let response = request::patch(
        "http://localhost:3000/foos/1",
        headers,
        &data,
        &app_router()
    );

    assert_json_api_error(
        response,
        JsonApiError {
            title: "from request fail".to_string(),
            detail: "from request fail".to_string(),
            status: "500".to_string()
        }
    );
}

#[test]
fn parse_json_api_custom_failure_in_get() {
    let response = request::get(
        "http://localhost:3000/foos/fail",
        Headers::new(),
        &app_router()
    );

    assert_json_api_error(
        response,
        JsonApiError {
            title: "fail in get".to_string(),
            detail: "fail in get".to_string(),
            status: "418".to_string()
        }
    );
}

#[test]
fn parse_json_api_custom_failure_in_index() {
    let response = request::get(
        "http://localhost:3000/foos?fail=yes",
        Headers::new(),
        &app_router()
    );

    assert_json_api_error(
        response,
        JsonApiError {
            title: "fail in index".to_string(),
            detail: "fail in index".to_string(),
            status: "418".to_string()
        }
    );
}

#[test]
fn parse_json_api_custom_failure_in_delete() {
    let response = request::delete(
        "http://localhost:3000/foos/fail",
        Headers::new(),
        &app_router()
    );

    assert_json_api_error(
        response,
        JsonApiError {
            title: "fail in delete".to_string(),
            detail: "fail in delete".to_string(),
            status: "418".to_string()
        }
    );
}

#[test]
fn parse_json_api_custom_failure_in_post() {
    let content_type: Mime = "application/vnd.api+json".parse().unwrap();

    let data = r#"
    {
        "data": {
            "id": "fail",
            "type": "foos",
            "attributes": {
                "title": "test"
            }
        }
    }"#;

    let mut headers = Headers::new();
    headers.set::<ContentType>(ContentType(content_type));

    let response = request::post("http://localhost:3000/foos", headers, &data, &app_router());

    assert_json_api_error(
        response,
        JsonApiError {
            title: "fail in post".to_string(),
            detail: "fail in post".to_string(),
            status: "418".to_string()
        }
    );
}

#[test]
fn parse_json_api_custom_failure_in_patch() {
    let content_type: Mime = "application/vnd.api+json".parse().unwrap();

    let data = r#"
    {
        "data": {
            "id": "fail",
            "type": "foos",
            "attributes": {
                "title": "test"
            }
        }
    }"#;

    let mut headers = Headers::new();
    headers.set::<ContentType>(ContentType(content_type));

    let response = request::patch(
        "http://localhost:3000/foos/fail",
        headers,
        &data,
        &app_router()
    );

    assert_json_api_error(
        response,
        JsonApiError {
            title: "fail in patch".to_string(),
            detail: "fail in patch".to_string(),
            status: "418".to_string()
        }
    )
}

#[test]
fn parse_json_api_failure_in_query_parse_in_get() {
    let response = request::get(
        "http://localhost:3000/foos/1?sort=fail",
        Headers::new(),
        &app_router()
    );

    assert_json_api_error(
        response,
        JsonApiError {
            title: "miscellaneous failure".to_string(),
            detail: "failed with reason: Query string parse error: Invalid value: fail".to_string(),
            status: "400".to_string()
        }
    );
}

#[test]
fn parse_json_api_failure_in_query_parse_in_index() {
    let response = request::get(
        "http://localhost:3000/foos?sort=fail",
        Headers::new(),
        &app_router()
    );

    assert_json_api_error(
        response,
        JsonApiError {
            title: "miscellaneous failure".to_string(),
            detail: "failed with reason: Query string parse error: Invalid value: fail".to_string(),
            status: "400".to_string()
        }
    );
}

#[test]
fn parse_no_json_in_post() {
    let response = request::post(
        "http://localhost:3000/foos",
        Headers::new(),
        "",
        &app_router()
    );

    assert_json_api_error(
        response,
        JsonApiError {
            detail: "No body".to_string(),
            status: "400".to_string(),
            title: "No body".to_string()
        }
    );
}

#[test]
fn parse_no_json_in_patch() {
    let response = request::patch(
        "http://localhost:3000/foos/1",
        Headers::new(),
        "",
        &app_router()
    );

    assert_json_api_error(
        response,
        JsonApiError {
            detail: "No body".to_string(),
            status: "400".to_string(),
            title: "No body".to_string()
        }
    );
}

#[test]
fn parse_invalid_json_in_post() {
    let content_type: Mime = "application/vnd.api+json".parse().unwrap();

    let data = r#"
    {
        "data": {
            "type": "foos",
            "attributes": {
                "published": "fail"
            }
        }
    }"#;

    let mut headers = Headers::new();
    headers.set::<ContentType>(ContentType(content_type));

    let response = request::post("http://localhost:3000/foos", headers, data, &app_router());

    assert_json_api_error(
        response,
        JsonApiError {
            title: "Can't parse body to the struct".to_string(),
            detail: "Can't parse body to the struct".to_string(),
            status: "400".to_string()
        }
    );
}

#[test]
fn parse_invalid_json_in_patch() {
    let content_type: Mime = "application/vnd.api+json".parse().unwrap();

    let data = r#"
    {
        "data": {
            "type": "foos",
            "attributes": {
                "published": "fail"
            }
        }
    }"#;

    let mut headers = Headers::new();
    headers.set::<ContentType>(ContentType(content_type));

    let response = request::patch("http://localhost:3000/foos/1", headers, data, &app_router());

    assert_json_api_error(
        response,
        JsonApiError {
            title: "Can't parse body to the struct".to_string(),
            detail: "Can't parse body to the struct".to_string(),
            status: "400".to_string()
        }
    );
}

fn assert_json_api_error(response: Result<Response, IronError>, error: JsonApiError) {
    let json = response::extract_body_to_string(response.unwrap());
    let result: JsonApiErrorArray = serde_json::from_str(&json).unwrap();

    let expected = JsonApiErrorArray {
        errors: vec![error]
    };

    assert_eq!(expected, result);
}
