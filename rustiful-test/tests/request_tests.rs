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
use iron::prelude::*;
use iron_test::{request, response};
use rustiful::FromRequest;

use rustiful::JsonApiArray;
use rustiful::JsonApiError;
use rustiful::JsonApiErrorArray;
use rustiful::JsonApiObject;
use rustiful::JsonApiResource;
use rustiful::JsonDelete;
use rustiful::JsonGet;
use rustiful::JsonIndex;
use rustiful::JsonPost;

use rustiful::ToJson;
use rustiful::status::Status;
use std::error::Error;
use std::fmt::Display;
use rustiful::iron::JsonApiRouterBuilder;
use std::fmt::Formatter;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonApi)]
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
        Ok(FooService {})
    }
}

#[derive(Debug)]
struct TestError(String);

impl Error for TestError {
    fn description(&self) -> &str {
        "fail"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl Display for TestError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "fail")
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
            -> Result<Option<Self>, Self::Error> {

        if id == "fail" {
            Err(TestError("test fail".to_string()))
        } else {
            Ok(Some(Foo {
                id: "1".to_string(),
                body: "test".to_string(),
                title: "test".to_string(),
                published: true,
            }))
        }
    }
}

impl JsonIndex for Foo {
    type Error = TestError;
    type Context = FooService;

    fn find_all(params: &Self::Params, ctx: Self::Context) -> Result<Vec<Self>, Self::Error> {
        Ok(vec![Foo {
                    id: "1".to_string(),
                    body: "test".to_string(),
                    title: "test".to_string(),
                    published: true,
                }])
    }
}

impl JsonDelete for Foo {
    type Error = TestError;
    type Context = FooService;

    fn delete(id: Self::JsonApiIdType, ctx: Self::Context) -> Result<(), Self::Error> {
        Err(TestError("fail".to_string()))
    }
}

impl JsonPost for Foo {
    type Error = TestError;
    type Context = FooService;

    fn create(id: Self::Resource, ctx: Self::Context) -> Result<Self, Self::Error> {
        Ok(Foo {
            id: "1".to_string(),
            body: "test".to_string(),
            title: "test".to_string(),
            published: true,
        })
    }
}

fn app_router() -> iron::Chain {
    let mut router = JsonApiRouterBuilder::default();
    router.jsonapi_get::<Foo>();
    router.jsonapi_post::<Foo>();
    router.jsonapi_index::<Foo>();
    router.jsonapi_delete::<Foo>();
    router.build()
}

#[test]
fn parse_json_api_index_get() {
    let headers = Headers::new();
    let response = request::get("http://localhost:3000/foos", headers, &app_router()).unwrap();
    let headers = response.headers.clone();
    let content_type = headers.get::<ContentType>().expect("no content type found!");
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
fn parse_json_api_custom_failure() {
    let response = request::get("http://localhost:3000/foos/fail",
                                Headers::new(),
                                &app_router());
    let result = response::extract_body_to_string(response.unwrap());
    let record: JsonApiErrorArray = serde_json::from_str(&result).unwrap();

    let expected = JsonApiErrorArray {
        errors: vec![JsonApiError {
                         detail: "fail".to_string(),
                         status: "418".to_string(),
                         title: "fail".to_string(),
                     }],
    };

    assert_eq!(expected, record);
}
