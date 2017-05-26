#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate rustiful_derive;

extern crate iron;
extern crate iron_test;
extern crate uuid;
extern crate rustiful;
extern crate serde_json;

use iron::Headers;
use iron::headers::ContentType;
use iron::mime::Mime;
use iron::prelude::*;
use iron_test::{request, response};
use rustiful::*;
use rustiful::JsonApiData;
use rustiful::iron::JsonApiRouterBuilder;
use rustiful::status::Status;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonApi)]
struct Foo {
    id: String,
    title: String,
    body: String,
    published: bool,
}

struct FooService;

impl FromRequest for FooService {
    type Error = TestError;
    fn from_request(request: &Request) -> Result<Self, Self::Error> {
        if let Some(_) = request.headers.get_raw("test-fail") {
            return Err(TestError("from request fail".to_string()));
        }
        Ok(FooService {})
    }
}

#[derive(Debug)]
struct TestError(String);

impl Error for TestError {
    fn description(&self) -> &str {
        &self.0
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl Display for TestError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> From<&'a TestError> for Status {
    fn from(error: &'a TestError) -> Self {
        rustiful::status::ImATeapot
    }
}

impl JsonGet for Foo {
    type Error = TestError;
    type Context = FooService;

    fn find(id: Self::JsonApiIdType,
            params: &Self::Params,
            ctx: Self::Context)
            -> Result<Option<JsonApiData<Self::Attrs>>, Self::Error> {

        if id == "fail" {
            Err(TestError("fail in get".to_string()))
        } else {
            Ok(Some(Foo {
                            id: "1".to_string(),
                            body: "test".to_string(),
                            title: "test".to_string(),
                            published: true,
                        }
                        .into_json(params)))
        }
    }
}

impl JsonIndex for Foo {
    type Error = TestError;
    type Context = FooService;

    fn find_all(params: &Self::Params,
                ctx: Self::Context)
                -> Result<Vec<JsonApiData<Self::Attrs>>, Self::Error> {
        if let Some(_) = params.query_params.get("fail") {
            return Err(TestError("fail in index".to_string()));
        }

        Ok(vec![(Foo {
                     id: "1".to_string(),
                     body: "test".to_string(),
                     title: "test".to_string(),
                     published: true,
                 },
                 params)
                        .into()])
    }
}

impl JsonDelete for Foo {
    type Error = TestError;
    type Context = FooService;

    fn delete(id: Self::JsonApiIdType, ctx: Self::Context) -> Result<(), Self::Error> {
        if id == "fail" {
            Err(TestError("fail in delete".to_string()))
        } else {
            Ok(())
        }
    }
}

impl JsonPost for Foo {
    type Error = TestError;
    type Context = FooService;

    fn create(json: JsonApiData<Self::Attrs>,
              ctx: Self::Context)
              -> Result<JsonApiData<Self::Attrs>, Self::Error> {
        if let Some(id) = json.id {
            if id == "fail" {
                return Err(TestError("fail in post".to_string()));
            }
        }

        Ok(Foo {
                   id: "1".to_string(),
                   body: "test".to_string(),
                   title: "test".to_string(),
                   published: true,
               }
               .into_json(&Default::default()))
    }
}

impl JsonPatch for Foo {
    type Error = TestError;
    type Context = FooService;

    fn update(id: Self::JsonApiIdType,
              json: JsonApiData<Self::Attrs>,
              ctx: Self::Context)
              -> Result<JsonApiData<Self::Attrs>, Self::Error> {
        if let Some(id) = json.id {
            if id == "fail" {
                return Err(TestError("fail in patch".to_string()));
            }
        }

        Ok(Foo {
                   id: "1".to_string(),
                   body: "test".to_string(),
                   title: "test".to_string(),
                   published: true,
               }
               .into_json(&Default::default()))
    }
}

fn app_router() -> iron::Chain {
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
    let records: JsonApiArray<<Foo as ToJson>::Resource> = serde_json::from_str(&result).unwrap();
    let params = <Foo as JsonApiResource>::from_str("").expect("failed to unwrap params");

    let test = Foo {
        id: "1".to_string(),
        body: "test".to_string(),
        title: "test".to_string(),
        published: true,
    };
    let data: <Foo as ToJson>::Resource = (test, &params).into();
    let expected: JsonApiArray<<Foo as ToJson>::Resource> = JsonApiArray { data: vec![data] };

    assert_eq!(expected, records);
}

#[test]
fn parse_json_api_index_get_with_fieldset() {
    let headers = Headers::new();
    let response = request::get("http://localhost:3000/foos?fields[foo]=title",
                                headers,
                                &app_router())
            .unwrap();
    let headers = response.headers.clone();
    let content_type = headers
        .get::<ContentType>()
        .expect("no content type found!");
    let result = response::extract_body_to_string(response);
    let records: JsonApiArray<<Foo as ToJson>::Resource> = serde_json::from_str(&result).unwrap();
    let params = <Foo as JsonApiResource>::from_str("").expect("failed to unwrap params");

    let test = Foo {
        id: "1".to_string(),
        body: "test".to_string(),
        title: "test".to_string(),
        published: true,
    };
    let data = JsonApiData::new(Some("1"),
                                "foo",
                                <Foo as ToJson>::Attrs::new(Some("test".to_string()), None, None));
    let expected: JsonApiArray<<Foo as ToJson>::Resource> = JsonApiArray { data: vec![data] };

    assert_eq!(expected, records);
}


#[test]
fn parse_json_api_single_get() {
    let response = request::get("http://localhost:3000/foos/1",
                                Headers::new(),
                                &app_router());
    let result = response::extract_body_to_string(response.unwrap());
    let record: JsonApiObject<<Foo as ToJson>::Attrs> = serde_json::from_str(&result).unwrap();
    let params = <Foo as JsonApiResource>::from_str("").expect("failed to unwrap params");

    let test = Foo {
        id: "1".to_string(),
        body: "test".to_string(),
        title: "test".to_string(),
        published: true,
    };
    let data: <Foo as ToJson>::Resource = (test, &params).into();
    let expected: JsonApiObject<<Foo as ToJson>::Attrs> = JsonApiObject { data: data };

    assert_eq!(expected, record);
}

#[test]
fn parse_json_api_single_get_fail_in_from_request() {
    let mut headers = Headers::new();
    headers.set_raw("test-fail", vec![]);

    let response = request::get("http://localhost:3000/foos/1", headers, &app_router());

    assert_json_api_error(response,
                          JsonApiError {
                              title: "from request fail".to_string(),
                              detail: "From request error: from request fail".to_string(),
                              status: "500".to_string(),
                          });
}

#[test]
fn parse_json_api_index_get_fail_in_from_request() {
    let mut headers = Headers::new();
    headers.set_raw("test-fail", vec![]);

    let response = request::get("http://localhost:3000/foos", headers, &app_router());

    assert_json_api_error(response,
                          JsonApiError {
                              title: "from request fail".to_string(),
                              detail: "From request error: from request fail".to_string(),
                              status: "500".to_string(),
                          });
}

#[test]
fn parse_json_api_delete_fail_in_from_request() {
    let mut headers = Headers::new();
    headers.set_raw("test-fail", vec![]);

    let response = request::delete("http://localhost:3000/foos/2", headers, &app_router());

    assert_json_api_error(response,
                          JsonApiError {
                              title: "from request fail".to_string(),
                              detail: "From request error: from request fail".to_string(),
                              status: "500".to_string(),
                          });
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

    assert_json_api_error(response,
                          JsonApiError {
                              title: "from request fail".to_string(),
                              detail: "From request error: from request fail".to_string(),
                              status: "500".to_string(),
                          });
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

    let response = request::patch("http://localhost:3000/foos/1",
                                  headers,
                                  &data,
                                  &app_router());

    assert_json_api_error(response,
                          JsonApiError {
                              title: "from request fail".to_string(),
                              detail: "From request error: from request fail".to_string(),
                              status: "500".to_string(),
                          });
}

#[test]
fn parse_json_api_custom_failure_in_get() {
    let response = request::get("http://localhost:3000/foos/fail",
                                Headers::new(),
                                &app_router());

    assert_json_api_error(response,
                          JsonApiError {
                              title: "fail in get".to_string(),
                              detail: "Error in repository: fail in get".to_string(),
                              status: "418".to_string(),
                          });
}

#[test]
fn parse_json_api_custom_failure_in_index() {
    let response = request::get("http://localhost:3000/foos?fail=yes",
                                Headers::new(),
                                &app_router());

    assert_json_api_error(response,
                          JsonApiError {
                              title: "fail in index".to_string(),
                              detail: "Error in repository: fail in index".to_string(),
                              status: "418".to_string(),
                          });
}

#[test]
fn parse_json_api_custom_failure_in_delete() {
    let response = request::delete("http://localhost:3000/foos/fail",
                                   Headers::new(),
                                   &app_router());

    assert_json_api_error(response,
                          JsonApiError {
                              title: "fail in delete".to_string(),
                              detail: "Error in repository: fail in delete".to_string(),
                              status: "418".to_string(),
                          });
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

    assert_json_api_error(response,
                          JsonApiError {
                              title: "fail in post".to_string(),
                              detail: "Error in repository: fail in post".to_string(),
                              status: "418".to_string(),
                          });
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

    let response = request::patch("http://localhost:3000/foos/fail",
                                  headers,
                                  &data,
                                  &app_router());

    assert_json_api_error(response,
                          JsonApiError {
                              title: "fail in patch".to_string(),
                              detail: "Error in repository: fail in patch".to_string(),
                              status: "418".to_string(),
                          })
}

#[test]
fn parse_json_api_failure_in_query_parse_in_get() {
    let response = request::get("http://localhost:3000/foos/1?sort=fail",
                                Headers::new(),
                                &app_router());

    assert_json_api_error(response,
                          JsonApiError {
                              title: "fail".to_string(),
                              detail: "Query string parse error:  Invalid value: fail".to_string(),
                              status: "400".to_string(),
                          });
}

#[test]
fn parse_json_api_failure_in_query_parse_in_index() {
    let response = request::get("http://localhost:3000/foos?sort=fail",
                                Headers::new(),
                                &app_router());

    assert_json_api_error(response,
                          JsonApiError {
                              title: "fail".to_string(),
                              detail: "Query string parse error:  Invalid value: fail".to_string(),
                              status: "400".to_string(),
                          });
}

#[test]
fn parse_no_json_in_post() {
    let response = request::post("http://localhost:3000/foos",
                                 Headers::new(),
                                 "",
                                 &app_router());

    assert_json_api_error(response,
                          JsonApiError {
                              detail: "No body".to_string(),
                              status: "400".to_string(),
                              title: "No body".to_string(),
                          });
}

#[test]
fn parse_no_json_in_patch() {
    let response = request::patch("http://localhost:3000/foos/1",
                                  Headers::new(),
                                  "",
                                  &app_router());

    assert_json_api_error(response,
                          JsonApiError {
                              detail: "No body".to_string(),
                              status: "400".to_string(),
                              title: "No body".to_string(),
                          });
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

    assert_json_api_error(response,
                          JsonApiError {
                              detail: "Error when parsing json: Can't parse body to the struct"
                                  .to_string(),
                              status: "400".to_string(),
                              title: "Can't parse body to the struct".to_string(),
                          });
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

    assert_json_api_error(response,
                          JsonApiError {
                              detail: "Error when parsing json: Can't parse body to the struct"
                                  .to_string(),
                              status: "400".to_string(),
                              title: "Can't parse body to the struct".to_string(),
                          });
}

fn assert_json_api_error(response: Result<Response, IronError>, error: JsonApiError) {
    let json = response::extract_body_to_string(response.unwrap());
    let result: JsonApiErrorArray = serde_json::from_str(&json).unwrap();

    let expected = JsonApiErrorArray { errors: vec![error] };

    assert_eq!(expected, result);
}
