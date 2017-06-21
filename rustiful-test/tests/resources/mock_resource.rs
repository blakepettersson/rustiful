use rustiful::*;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonApi)]
pub struct Foo {
    pub id: String,
    pub title: String,
    pub body: String,
    pub published: bool,
}

pub struct FooService;

#[derive(Debug)]
pub struct TestError(pub String);

impl Error for TestError {
    fn description(&self) -> &str {
        &self.0
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl Display for TestError {
    fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl JsonGet for Foo {
    type Error = TestError;
    type Context = FooService;

    fn find(
        id: Self::JsonApiIdType,
        params: &Self::Params,
        _: Self::Context,
    ) -> Result<Option<JsonApiData<Self>>, Self::Error> {

        if id == "fail" {
            Err(TestError("fail in get".to_string()))
        } else {
            Ok(Some(
                Foo {
                    id: "1".to_string(),
                    body: "test".to_string(),
                    title: "test".to_string(),
                    published: true,
                }.into_json(params),
            ))
        }
    }
}

impl JsonIndex for Foo {
    type Error = TestError;
    type Context = FooService;

    fn find_all(
        params: &Self::Params,
        _: Self::Context,
    ) -> Result<Vec<JsonApiData<Self>>, Self::Error> {
        if let Some(_) = params.query_params.get("fail") {
            return Err(TestError("fail in index".to_string()));
        }

        Ok(vec![
            (
                Foo {
                    id: "1".to_string(),
                    body: "test".to_string(),
                    title: "test".to_string(),
                    published: true,
                },
                params
            ).into(),
        ])
    }
}

impl JsonDelete for Foo {
    type Error = TestError;
    type Context = FooService;

    fn delete(id: Self::JsonApiIdType, _: Self::Context) -> Result<(), Self::Error> {
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

    fn create(
        json: JsonApiData<Self>,
        params: &Self::Params,
        _: Self::Context,
    ) -> Result<JsonApiData<Self>, Self::Error> {
        if let Some(id) = json.id {
            if id == "fail" {
                return Err(TestError("fail in post".to_string()));
            }
        }

        Ok(
            Foo {
                id: "1".to_string(),
                body: "test".to_string(),
                title: "test".to_string(),
                published: true,
            }.into_json(params),
        )
    }
}

impl JsonPatch for Foo {
    type Error = TestError;
    type Context = FooService;

    fn update(
        _: Self::JsonApiIdType,
        json: JsonApiData<Self>,
        params: &Self::Params,
        _: Self::Context,
    ) -> Result<JsonApiData<Self>, Self::Error> {
        if let Some(id) = json.id {
            if id == "fail" {
                return Err(TestError("fail in patch".to_string()));
            }
        }

        Ok(
            Foo {
                id: "1".to_string(),
                body: "test".to_string(),
                title: "test".to_string(),
                published: true,
            }.into_json(params),
        )
    }
}
