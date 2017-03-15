#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate jsonapi_derive;

extern crate iron;
extern crate iron_test;
extern crate uuid;
extern crate jsonapi;
extern crate hyper;
extern crate serde_json;

use jsonapi::jsonapi_array::JsonApiArray;
use std::str::FromStr;
use std::net::ToSocketAddrs;
use iron::Url;
use iron::Request;
use iron::typemap::TypeMap;
use iron::method::Method;
use uuid::Uuid;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use jsonapi::queryspec::ToParams;
use jsonapi::service::JsonApiService;

use iron::{Handler, Headers, status};
use iron::IronResult;
use iron::Response;
use iron::mime::Mime;
use iron_test::{request, response};

use jsonapi::queryspec::ToJson;
use iron::prelude::*;

struct IndexHandler;

impl iron::Handler for IndexHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        self::__fooservice::routes::index(req)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonApi)]
struct Foo {
    id: String,
    title: String,
    body: String,
    published: bool,
}

#[derive(JsonApiRepository)]
#[resource="tests"]
struct FooService;

impl FooService {
    fn new() -> FooService {
        FooService {}
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

impl JsonApiService for FooService {
    type T = Foo;
    type Error = TestError;

    fn find(&self, id: &str, _: &<Foo as ToParams>::Params) -> Result<Option<Foo>, Self::Error> {
        Ok(Some(Foo {
            id: "1".to_string(),
            body: "test".to_string(),
            title: "test".to_string(),
            published: true
        }))
    }

    fn find_all(&self, params: &<Foo as ToParams>::Params) -> Result<Vec<Foo>, Self::Error> {
        Ok(vec![Foo {
            id: "1".to_string(),
            body: "test".to_string(),
            title: "test".to_string(),
            published: true
        }])
    }

    fn save(&self, record: Foo) -> Result<Foo, Self::Error> {
        Err(TestError("fail".to_string()))
    }

    fn delete(&self, id: &str) -> Result<(), Self::Error> {
        Err(TestError("fail".to_string()))
    }
}

#[test]
fn parse_json_api_index_get() {
    let headers = Headers::new();
    let response = request::get("http://test.com", headers, &IndexHandler);
    let result = response::extract_body_to_string(response.unwrap());

    let records: JsonApiArray<<Foo as ToJson>::Json> = serde_json::from_str(&result).unwrap();
    let params = <Foo as ToParams>::Params::from_str("").expect("failed to unwrap params");

    let test = Foo {
        id: "1".to_string(),
        body: "test".to_string(),
        title: "test".to_string(),
        published: true
    };
    let data:<Foo as ToJson>::Json = (test, &params).into();
    let expected:JsonApiArray<<Foo as ToJson>::Json> = JsonApiArray {
        data: vec![data]
    };

    assert_eq!(expected, records);
}
