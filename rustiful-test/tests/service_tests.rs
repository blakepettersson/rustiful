#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate rustiful_derive;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_codegen;

#[macro_use]
extern crate lazy_static;

extern crate r2d2;
extern crate r2d2_diesel;
extern crate uuid;
extern crate rustiful;
extern crate iron;
extern crate dotenv;

use self::iron::prelude::*;
use diesel::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use r2d2::GetTimeout;
use r2d2::Pool;
use r2d2::PooledConnection;
use r2d2_diesel::ConnectionManager;
use rustiful::*;
use rustiful::SortOrder::*;
use rustiful::status::Status;
use std::env;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use uuid::Uuid;

infer_schema!("dotenv:DATABASE_URL");

use self::tests as column;
use self::tests::dsl::tests as table;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonApi, Queryable, Insertable,
AsChangeset)]
#[table_name="tests"]
struct Test {
    id: String,
    title: String,
    body: Option<String>,
    published: bool,
}

#[derive(Debug)]
enum MyErr {
    Diesel(diesel::result::Error),
    UpdateError(String),
}

impl Error for MyErr {
    fn description(&self) -> &str {
        match *self {
            MyErr::Diesel(ref err) => err.description(),
            MyErr::UpdateError(ref err) => err,
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            MyErr::Diesel(ref err) => err.cause(),
            MyErr::UpdateError(_) => None,
        }
    }
}

impl Display for MyErr {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match *self {
            MyErr::Diesel(ref err) => err.fmt(f),
            MyErr::UpdateError(ref err) => err.fmt(f),
        }
    }
}

impl FromRequest for DB {
    type Error = GetTimeout;

    fn from_request(_: &Request) -> Result<DB, Self::Error> {
        match DB_POOL.get() {
            Ok(conn) => Ok(DB(conn)),
            Err(e) => Err(e),
        }
    }
}

impl<'a> From<&'a MyErr> for Status {
    fn from(_: &'a MyErr) -> Self {
        rustiful::status::InternalServerError
    }
}

impl JsonGet for Test {
    type Error = MyErr;
    type Context = DB;

    fn find(id: Self::JsonApiIdType,
            _: &Self::Params,
            ctx: Self::Context)
            -> Result<Option<Self>, Self::Error> {
        table.find(id).first(ctx.conn()).optional().map_err(|e| MyErr::Diesel(e))
    }
}

impl JsonPatch for Test {
    type Error = MyErr;
    type Context = DB;

    fn update(id: Self::JsonApiIdType,
              json: Self::Resource,
              ctx: Self::Context)
              -> Result<Self, Self::Error> {
        let record = table.find(&id).first(ctx.conn()).map_err(|e| MyErr::Diesel(e))?;
        let patch = (record, json).try_into().map_err(|e| MyErr::UpdateError(e))?;
        diesel::update(table.find(&id)).set(&patch)
            .execute(ctx.conn())
            .map_err(|e| MyErr::Diesel(e))?;
        Ok(patch)
    }
}

impl JsonPost for Test {
    type Error = MyErr;
    type Context = DB;

    fn create(record: Self::Resource, ctx: Self::Context) -> Result<Self, Self::Error> {
        let result: Test = record.into();
        diesel::insert(&result)
            .into(table)
            .execute(ctx.conn())
            .map_err(|e| MyErr::Diesel(e))
            .map(|_| result)
    }
}

impl JsonIndex for Test {
    type Error = MyErr;
    type Context = DB;

    fn find_all(params: &Self::Params, ctx: Self::Context) -> Result<Vec<Self>, Self::Error> {
        let mut query = table.into_boxed();

        {
            use self::test::sort::*;
            for order in &params.sort.fields {
                match *order {
                    title(Asc) => {
                        query = query.order(column::title);
                    }
                    title(Desc) => {
                        query = query.order(column::title.desc());
                    }
                    body(Asc) => {
                        query = query.order(column::body);
                    }
                    body(Desc) => {
                        query = query.order(column::body.desc());
                    }
                    published(Asc) => {
                        query = query.order(column::published);
                    }
                    published(Desc) => {
                        query = query.order(column::published.desc());
                    }
                };
            }
        }

        query.load(ctx.conn()).map_err(|e| MyErr::Diesel(e))
    }
}

impl JsonDelete for Test {
    type Error = MyErr;
    type Context = DB;

    fn delete(id: Self::JsonApiIdType, ctx: Self::Context) -> Result<(), Self::Error> {
        diesel::delete(table.find(id)).execute(ctx.conn()).map(|_| ()).map_err(|e| MyErr::Diesel(e))
    }
}


impl From<<Test as ToJson>::Resource> for Test {
    fn from(json: <Test as ToJson>::Resource) -> Self {
        Test {
            id: json.id.map(|id| id.into()).unwrap_or_else(|| Uuid::new_v4().to_string()),
            title: json.attributes.title.unwrap_or("".to_string()),
            body: json.attributes.body.unwrap_or(None),
            published: json.attributes.published.unwrap_or(false),
        }
    }
}

lazy_static! {
    pub static ref DB_POOL: Pool<ConnectionManager<SqliteConnection>> = create_db_pool();
}

pub struct DB(PooledConnection<ConnectionManager<SqliteConnection>>);

impl DB {
    pub fn conn(&self) -> &SqliteConnection {
        &*self.0
    }
}

pub fn create_db_pool() -> Pool<ConnectionManager<SqliteConnection>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let config = r2d2::Config::default();
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::new(config, manager).expect("Failed to create pool.")
}

#[test]
fn test_crud() {
    let id = Uuid::new_v4().to_string();
    let model = Test {
        id: id.clone(),
        title: "1".to_string(),
        body: Some("1".to_string()),
        published: true,
    };

    Test::create(model.clone().into(),
                 DB(DB_POOL.get().expect("cannot get connection")))
        .unwrap();
    let params = <Test as JsonApiResource>::from_str("").unwrap();
    assert_eq!(model,
               Test::find(id.clone(),
                          &params,
                          DB(DB_POOL.get().expect("cannot get connection")))
                   .unwrap()
                   .unwrap());

    let json_attrs = <Test as ToJson>::Attrs::new(Some("3".to_string()), None, None);
    let json = JsonApiData::new(Some(id.clone()), "".to_string(), json_attrs);
    Test::update(id.clone(),
                 json,
                 DB(DB_POOL.get().expect("cannot get connection")));

    let updated = Test::find(id.clone(),
                             &params,
                             DB(DB_POOL.get().expect("cannot get connection")))
        .unwrap()
        .unwrap();

    assert_eq!(updated.body, Some("1".to_string()));
    assert_eq!(updated.title, "3".to_string());
    assert_eq!(updated.published, true);
}

#[test]
fn test_setting_of_id_in_try_from() {
    let json_attrs = <Test as ToJson>::Attrs::new(Some("3".to_string()), None, None);
    let json = JsonApiData::new(Some("1".to_string()), "".to_string(), json_attrs);
    let test = Test {
        id: "1".to_string(),
        title: "foo".to_string(),
        body: None,
        published: false,
    };

    let expected_id = test.id.clone();
    let result: Test = (test, json).try_into().unwrap();
    assert_eq!(expected_id, result.id)
}
