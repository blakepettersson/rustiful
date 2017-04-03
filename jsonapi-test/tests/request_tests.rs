#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate jsonapi_derive;

extern crate iron;
extern crate router;
extern crate iron_test;
extern crate uuid;
extern crate jsonapi;
extern crate serde_json;

use self::router::Router;

use jsonapi::array::JsonApiArray;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use jsonapi::query_string::QueryString;
use jsonapi::params::JsonApiResource;
use jsonapi::object::JsonApiObject;
use jsonapi::service::JsonGet;
use jsonapi::service::JsonIndex;
use jsonapi::service::JsonDelete;
use jsonapi::service::JsonApiService;
use std::marker::PhantomData;
use jsonapi::iron::DeleteRouter;
use jsonapi::iron::GetHandler;
use jsonapi::iron::IndexHandler;
use jsonapi::iron::DeleteHandler;
use iron::headers::ContentType;
use iron::Headers;
use jsonapi::iron::GetRouter;
use jsonapi::iron::IndexRouter;
use iron_test::{request, response};

use jsonapi::queryspec::ToJson;
use iron::prelude::*;

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

impl Default for FooService {
    fn default() -> Self {
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

impl JsonGet for Foo {
    type Error = TestError;
    type Context = FooService;

    fn find(id: Self::JsonApiIdType, params: &Self::Params, ctx: Self::Context) -> Result<Option<Self>, Self::Error> {
        Ok(Some(Foo {
            id: "1".to_string(),
            body: "test".to_string(),
            title: "test".to_string(),
            published: true
        }))
    }
}

impl JsonIndex for Foo {
    type Error = TestError;
    type Context = FooService;

    fn find(params: &Self::Params, ctx: Self::Context) -> Result<Vec<Self>, Self::Error> {
        Ok(vec![Foo {
            id: "1".to_string(),
            body: "test".to_string(),
            title: "test".to_string(),
            published: true
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

fn app_router() -> Router {
    let mut router = Router::new();
    router.jsonapi_get(PhantomData::<Foo>);
    router.jsonapi_index(PhantomData::<Foo>);
    router.jsonapi_delete(PhantomData::<Foo>);
    router
}

#[test]
fn parse_json_api_index_get() {
    let headers = Headers::new();
    let response = request::get("http://localhost:3000/foos", headers, &app_router()).unwrap();
    let headers = response.headers.clone();
    let content_type = headers.get::<ContentType>().expect("no content type found!");
    let result = response::extract_body_to_string(response);
    let records: JsonApiArray<<Foo as ToJson>::Resource> = serde_json::from_str(&result).unwrap();
    let params = <Foo as QueryString>::from_str("").expect("failed to unwrap params");

    let test = Foo {
        id: "1".to_string(),
        body: "test".to_string(),
        title: "test".to_string(),
        published: true
    };
    let data:<Foo as ToJson>::Resource = (test, &params).into();
    let expected:JsonApiArray<<Foo as ToJson>::Resource> = JsonApiArray {
        data: vec![data]
    };

    assert_eq!(expected, records);
}

#[test]
fn parse_json_api_single_get() {
    let response = request::get("http://localhost:3000/foos/1",
                                Headers::new(),
                                &app_router());
    let result = response::extract_body_to_string(response.unwrap());
    let record: JsonApiObject<<Foo as ToJson>::Resource> = serde_json::from_str(&result).unwrap();
    let params = <Foo as QueryString>::from_str("").expect("failed to unwrap params");

    let test = Foo {
        id: "1".to_string(),
        body: "test".to_string(),
        title: "test".to_string(),
        published: true
    };
    let data:<Foo as ToJson>::Resource = (test, &params).into();
    let expected:JsonApiObject<<Foo as ToJson>::Resource> = JsonApiObject {
        data: data
    };

    assert_eq!(expected, record);
}