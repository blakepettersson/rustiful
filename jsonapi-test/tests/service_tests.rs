#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate jsonapi_derive;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_codegen;

extern crate uuid;

extern crate jsonapi;

use uuid::Uuid;
use jsonapi::SortOrder::*;
use diesel::*;
use jsonapi::*;

type TestConnection = ::diesel::sqlite::SqliteConnection;

infer_schema!("/tmp/test.db");

use self::tests::dsl::tests as table;
use self::tests as column;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonApi, Queryable, Insertable)]
#[table_name="tests"]
struct Test {
    id: String,
    title: String,
    body: String,
    published: bool,
}

struct TestService;

impl Default for TestService {
    fn default() -> Self {
        TestService {}
    }
}

impl JsonGet for Test {
    type Error = diesel::result::Error;
    type Context = TestService;

    fn find(id: Self::JsonApiIdType,
            params: &Self::Params,
            ctx: Self::Context)
            -> Result<Option<Self>, Self::Error> {
        table.find(id).first(&connection()).optional()
    }
}

impl JsonPost for Test {
    type Error = diesel::result::Error;
    type Context = TestService;

    fn create(record: Self::Resource, ctx: Self::Context) -> Result<Self, Self::Error> {
        //diesel::insert(&record).into(table).execute(&connection()).map(|_| record)
        // TODO: Add try_into for test
        Ok(Test {
            id: "1".to_string(),
            title: "1".to_string(),
            body: "1".to_string(),
            published: true
        })
    }
}

impl JsonIndex for Test {
    type Error = diesel::result::Error;
    type Context = TestService;

    fn find_all(params: &Self::Params, ctx: Self::Context) -> Result<Vec<Self>, Self::Error> {
        use self::test::sort::*;

        let mut query = table.into_boxed();

        for order in &params.sort.fields {
            match order {
                &title(Asc) => {
                    query = query.order(column::title);
                }
                &title(Desc) => {
                    query = query.order(column::title.desc());
                }
                &body(Asc) => {
                    query = query.order(column::body);
                }
                &body(Desc) => {
                    query = query.order(column::body.desc());
                }
                &published(Asc) => {
                    query = query.order(column::published);
                }
                &published(Desc) => {
                    query = query.order(column::published.desc());
                }
            };
        }

        query.load(&connection())
    }
}

impl JsonDelete for Test {
    type Error = diesel::result::Error;
    type Context = TestService;

    fn delete(id: Self::JsonApiIdType, ctx: Self::Context) -> Result<(), Self::Error> {
        diesel::delete(table.find(id)).execute(&connection()).map(|_| ())
    }
}


impl TryFrom<<Test as ToJson>::Resource> for Test {
    type Error = QueryStringParseError;

    fn try_from(json: <Test as ToJson>::Resource) -> Result<Self, Self::Error> {
        Err(QueryStringParseError::UnImplementedError)
    }
}

fn connection() -> TestConnection {
    let result = connection_without_transaction();
    result
}

fn connection_without_transaction() -> TestConnection {
    let connection = ::diesel::sqlite::SqliteConnection::establish("/tmp/test.db").unwrap();
    connection
}

#[test]
fn test() {
    let id = Uuid::new_v4().to_string();
    let model = Test {
        id: id.clone(),
        title: "1".to_string(),
        body: "1".to_string(),
        published: true,
    };
    let service = TestService {};
    let params = <Test as JsonApiResource>::from_str("").unwrap();
    Test::create(model.clone().into(), service).unwrap();
    assert_eq!(model, Test::find(id, &params, Default::default()).unwrap().unwrap());
}
