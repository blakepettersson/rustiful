use rustiful::*;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonApi)]
pub struct Foo {
    pub id: String,
    pub title: String,
    pub body: String,
    pub published: bool
}

impl Foo {
    fn new<Id: Into<String>, Title: Into<String>, Body: Into<String>>(
        id: Id,
        title: Title,
        body: Body,
        published: bool
    ) -> Self {
        Foo {
            id: id.into(),
            title: title.into(),
            body: body.into(),
            published: published
        }
    }
}

pub struct FooService;

#[derive(Debug)]
pub struct TestError(pub String);

impl TestError {
    fn new<S: Into<String>>(s: S) -> Self {
        TestError(s.into())
    }
}


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
        _: Self::Context
    ) -> Result<Option<JsonApiData<Self>>, (Self::Error, Self::Status)> {

        if id == "fail" {
            Err(TestError("fail in get".to_string()).into())
        } else {
            Ok(Some(Foo::new("1", "test", "test", true).into_json(params)))
        }
    }
}

impl JsonIndex for Foo {
    type Error = TestError;
    type Context = FooService;

    fn find_all(
        params: &Self::Params,
        _: Self::Context
    ) -> Result<Vec<JsonApiData<Self>>, (Self::Error, Self::Status)> {
        if let Some(_) = params.query_params.get("fail") {
            let err = TestError::new("fail in index");
            return Err(err.into());
        }

        Ok(vec![Foo::new("1", "test", "test", true)].into_json(params))
    }
}

impl JsonDelete for Foo {
    type Error = TestError;
    type Context = FooService;

    fn delete(
        id: Self::JsonApiIdType,
        _: Self::Context
    ) -> Result<(), (Self::Error, Self::Status)> {
        if id == "fail" {
            let err = TestError::new("fail in delete");
            return Err(err.into());
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
        _: Self::Context
    ) -> Result<JsonApiData<Self>, (Self::Error, Self::Status)> {
        if let Some(id) = json.id {
            if id == "fail" {
                let err = TestError::new("fail in post");
                return Err(err.into());
            }
        }

        Ok(Foo::new("1", "test", "test", true).into_json(params))
    }
}

impl JsonPatch for Foo {
    type Error = TestError;
    type Context = FooService;

    fn update(
        _: Self::JsonApiIdType,
        json: JsonApiData<Self>,
        params: &Self::Params,
        _: Self::Context
    ) -> Result<JsonApiData<Self>, (Self::Error, Self::Status)> {
        if let Some(id) = json.id {
            if id == "fail" {
                let err = TestError::new("fail in patch");
                return Err(err.into());
            }
        }

        Ok(Foo::new("1", "test", "test", true).into_json(params))
    }
}
