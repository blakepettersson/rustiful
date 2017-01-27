extern crate serde_json;
extern crate serde;

use queryspec::ToTypedQuerySpec;
use self::serde::ser::Serialize;
use self::serde::de::Deserialize;
use super::queryspec::QuerySpec;

pub trait JsonApiService<T, E, S = Self>
    where T: Serialize + Deserialize  + ToTypedQuerySpec
{
    //use hyper::{Get, Post, StatusCode};
    //use hyper::header::ContentLength;
    //use hyper::server::{Http, Service, Request, Response};

    fn find(id: &str, QuerySpec<T>) -> Result<Option<T>, E>;

    fn save(record: T) -> Result<T, E>;

    fn delete(id: &str) -> Result<(), E>;
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::super::queryspec::QuerySpec;
    use super::super::queryspec::ToTypedQuerySpec;
    use super::super::queryspec::QueryStringParseError;
    use super::super::service::JsonApiService;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct Foo {
        bar: Option<String>,
    }

    struct Bar {
        foo: Option<String>
    }

    struct TestService;
    impl JsonApiService<Foo, String> for TestService {
        fn find(id: &str, query_spec: QuerySpec<Foo>) -> Result<Option<Foo>, String> {
            Ok(Some(Foo { bar: query_spec.fields().foo }))
        }

        fn save(record: Foo) -> Result<Foo, String> {
            Ok(Foo { bar: None })
        }

        fn delete(id: &str) -> Result<(), String> {
            Ok(())
        }
    }

    impl ToTypedQuerySpec for Foo {
        type T = Bar;
        fn fields() -> Bar {
            Bar {
                foo: Some("bla".to_string())
            }
        }
    }

    #[test]
    fn it_works() {
        let foo = Foo { bar: Some("bla".to_string()) };
        let query_spec = QuerySpec::new(HashMap::new());

        let result = TestService::find(&"foo", query_spec).expect("nooo").expect("nooo");
        assert_eq!(foo, result);
    }
}
