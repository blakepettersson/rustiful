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
use jsonapi::sort_order::SortOrder::*;
use jsonapi::service::JsonApiService;
use jsonapi::params::JsonApiResource;
use jsonapi::queryspec::QueryStringParseError;
use diesel::*;
use std::str::FromStr;

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
    published: bool
}

#[derive(JsonApiRepository)]
#[resource="tests"]
struct TestService;

impl TestService {
    fn new() -> TestService {
        TestService {}
    }
}

//#[derive(JsonApiRepository)]
//struct Foo;

impl JsonApiService for TestService {
    type T = Test;
    type Error = diesel::result::Error;

    fn find(&self, id: &str, _: &<Test as JsonApiResource>::Params) -> Result<Option<Test>, Self::Error> {
        table.find(id).first(&connection()).optional()
    }

    fn find_all(&self, params: &<Test as JsonApiResource>::Params) -> Result<Vec<Test>, Self::Error> {
        use self::test::sort::*;

        let mut query = table.into_boxed();

        for order in &params.sort.fields {
            match order {
                &title(Asc) => { query = query.order(column::title); },
                &title(Desc) => { query = query.order(column::title.desc()); },
                &body(Asc) => { query = query.order(column::body); },
                &body(Desc) => { query = query.order(column::body.desc()); }
                &published(Asc) => { query = query.order(column::published); },
                &published(Desc) => { query = query.order(column::published.desc()); }
            };
        }

        query.load(&connection())
    }

    fn save(&self, record: Test) -> Result<Test, Self::Error> {
        diesel::insert(&record).into(table).execute(&connection()).map(|_| record)
    }

    fn delete(&self, id: &str) -> Result<(), Self::Error> {
        diesel::delete(table.find(id)).execute(&connection()).map(|_| ())
    }
}

fn connection() -> TestConnection {
    let result = connection_without_transaction();
    //result.begin_test_transaction().unwrap();
    result
}

fn connection_without_transaction() -> TestConnection {
    let connection = ::diesel::sqlite::SqliteConnection::establish("/tmp/test.db").unwrap();
    //let migrations_dir = ::diesel::migrations::find_migrations_directory().unwrap().join("sqlite");
    //::diesel::migrations::run_pending_migrations_in_directory(&connection, &migrations_dir, &mut io::sink()).unwrap();
    connection
}

#[test]
fn test() {
    let id = Uuid::new_v4().to_string();
    let model = Test {
        id: id.clone(),
        title: "1".to_string(),
        body: "1".to_string(),
        published: true
    };
    let service = TestService {};

    //let params = <Test as ToParams>::Params::from_str("").unwrap();
    //service.save(model.clone()).unwrap();
    //assert_eq!(model, service.find(&id, &params).unwrap().unwrap());
}