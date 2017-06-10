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

extern crate iron;
extern crate iron_test;
extern crate uuid;
extern crate rustiful;
extern crate serde_json;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate dotenv;

use diesel::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use iron::Headers;
use iron::headers::ContentType;
use iron::mime::Mime;
use iron::prelude::*;
use iron_test::{request, response};
use r2d2::GetTimeout;
use r2d2::Pool;
use r2d2::PooledConnection;
use r2d2_diesel::ConnectionManager;
use rustiful::FromRequest;
use rustiful::IntoJson;
use rustiful::JsonApiData;
use rustiful::JsonApiContainer;
use rustiful::JsonDelete;
use rustiful::JsonGet;
use rustiful::JsonIndex;
use rustiful::JsonPatch;
use rustiful::JsonPost;
use rustiful::SortOrder::*;
use rustiful::ToJson;
use rustiful::TryInto;
use rustiful::iron::JsonApiRouterBuilder;
use rustiful::status::Status;
use std::env;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use uuid::Uuid;

infer_schema!("dotenv:DATABASE_URL");

use self::tests as column;
use self::tests::dsl::tests as table;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonApi, Queryable,
Insertable, AsChangeset)]
#[table_name="tests"]
#[changeset_options(treat_none_as_null = "true")]
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
    fn from(err: &'a MyErr) -> Self {
        match *err {
            MyErr::UpdateError(_) => rustiful::status::ImATeapot,
            _ => rustiful::status::InternalServerError,
        }
    }
}

impl JsonGet for Test {
    type Error = MyErr;
    type Context = DB;

    fn find(id: Self::JsonApiIdType,
            params: &Self::Params,
            ctx: Self::Context)
            -> Result<Option<JsonApiData<Self>>, Self::Error> {
        if id == "fail" {
            return Err(MyErr::UpdateError("test fail".to_string()));
        }
        table
            .find(id)
            .first::<Test>(ctx.conn())
            .optional()
            .map(|r| r.map(|i| i.into_json(params)))
            .map_err(|e| MyErr::Diesel(e))
    }
}

impl JsonPatch for Test {
    type Error = MyErr;
    type Context = DB;

    fn update(id: Self::JsonApiIdType,
              json: JsonApiData<Self>,
              params: &Self::Params,
              ctx: Self::Context)
              -> Result<JsonApiData<Self>, Self::Error> {
        let record = table
            .find(&id)
            .first(ctx.conn())
            .map_err(|e| MyErr::Diesel(e))?;
        let patch = (record, json)
            .try_into()
            .map_err(|e| MyErr::UpdateError(e))?;
        diesel::update(table.find(&id))
            .set(&patch)
            .execute(ctx.conn())
            .map_err(|e| MyErr::Diesel(e))?;
        Ok(patch.into_json(params))
    }
}

impl JsonIndex for Test {
    type Error = MyErr;
    type Context = DB;

    fn find_all(params: &Self::Params,
                ctx: Self::Context)
                -> Result<Vec<JsonApiData<Self>>, Self::Error> {
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

        query
            .load::<Test>(ctx.conn())
            .map(|r| r.into_json(params))
            .map_err(|e| MyErr::Diesel(e))
    }
}

impl JsonDelete for Test {
    type Error = MyErr;
    type Context = DB;

    fn delete(id: Self::JsonApiIdType, ctx: Self::Context) -> Result<(), Self::Error> {
        diesel::delete(table.find(id))
            .execute(ctx.conn())
            .map(|_| ())
            .map_err(|e| MyErr::Diesel(e))
    }
}

impl JsonPost for Test {
    type Error = MyErr;
    type Context = DB;

    fn create(json: JsonApiData<Self>,
              params: &Self::Params,
              ctx: Self::Context)
              -> Result<JsonApiData<Self>, Self::Error> {
        let has_client_id = json.has_id(); // Client-supplied id
        let mut result: Test = json.try_into().map_err(|e| MyErr::UpdateError(e))?;

        // SQlite hack; instead of using auto-generated id, create a UUID if the id hasn't
        // already been supplied by the client.
        if !has_client_id {
            result.id = Uuid::new_v4().to_string();
        }

        diesel::insert(&result)
            .into(table)
            .execute(ctx.conn())
            .map_err(|e| MyErr::Diesel(e))
            .map(|_| result.into_json(params))
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

fn app_router() -> iron::Chain {
    let mut router = JsonApiRouterBuilder::default();
    router.jsonapi_get::<Test>();
    router.jsonapi_post::<Test>();
    router.jsonapi_index::<Test>();
    router.jsonapi_patch::<Test>();
    router.jsonapi_delete::<Test>();
    router.build()
}

#[test]
#[ignore] // Ignored by default since we need to run this sequentially, due to SQLite locking.
fn post_without_client_generated_id() {
    let data = r#"
    {
        "data": {
            "type": "tests",
            "attributes": {
                "body": "test",
                "title": "test",
                "published": true
            }
        }
    }"#;

    let created = do_post(&data);
    let retrieved = do_get(&created.clone().data.id.unwrap());

    assert_eq!(created, retrieved);
}

#[test]
#[ignore] // Ignored by default since we need to run this sequentially, due to SQLite locking.
fn post_with_client_generated_id() {
    let id = Uuid::new_v4().to_string();
    let data = format!(r#"
    {{
        "data": {{
            "id": "{}",
            "type": "tests",
            "attributes": {{
                "body": "test",
                "title": "test",
                "published": true
            }}
        }}
    }}"#,
                       id);

    let created = do_post(&data);
    let retrieved = do_get(&id);

    assert_eq!(created, retrieved);
}

#[test]
#[ignore] // Ignored by default since we need to run this sequentially, due to SQLite locking.
fn post_with_client_generated_id_and_fieldset_params() {
    let id = Uuid::new_v4().to_string();
    let data = format!(r#"
    {{
        "data": {{
            "id": "{}",
            "type": "tests",
            "attributes": {{
                "body": "test",
                "title": "test",
                "published": true
            }}
        }}
    }}"#,
                       id);

    let created = do_post_with_url(&data, "http://localhost:3000/tests?fields[tests]=title");
    let expected =
        JsonApiData::new(Some(id),
                         "tests",
                         <Test as ToJson>::Attrs::new(Some("test".to_string()), Some(None), None));

    assert_eq!(created.data, expected);
}

#[test]
#[ignore] // Ignored by default since we need to run this sequentially, due to SQLite locking.
fn update_with_nullable_field() {
    let id = Uuid::new_v4().to_string();
    let data = format!(r#"
    {{
        "data": {{
            "id": "{}",
            "type": "tests",
            "attributes": {{
                "body": "test",
                "title": "test",
                "published": true
            }}
        }}
    }}"#,
                       &id);

    do_post(&data);

    {
        let patch = format!(r#"
        {{
            "data": {{
                "id": "{}",
                "type": "tests",
                "attributes": {{
                    "title": "funky"
                }}
            }}
        }}"#,
                            &id);

        do_patch(&id, &patch);


        let retrieved = do_get(&id);
        assert_eq!(Some("test".to_string()),
                   retrieved.data.attributes.body.unwrap());
        assert_eq!(Some("funky".to_string()), retrieved.data.attributes.title);
    }

    {
        let patch = format!(r#"
        {{
            "data": {{
                "id": "{}",
                "type": "tests",
                "attributes": {{
                    "body": "new_content"
                }}
            }}
        }}"#,
                            &id);

        do_patch(&id, &patch);

        let retrieved = do_get(&id);
        assert_eq!(Some("new_content".to_string()),
                   retrieved.data.attributes.body.unwrap());
    }

    {
        let patch = format!(r#"
        {{
            "data": {{
                "id": "{}",
                "type": "tests",
                "attributes": {{
                    "body": null
                }}
            }}
        }}"#,
                            &id);

        do_patch(&id, &patch);

        let retrieved = do_get(&id);
        assert_eq!(None, retrieved.data.attributes.body.unwrap());
    }
}

#[test]
#[ignore] // Ignored by default since we need to run this sequentially, due to SQLite locking.
fn update_with_fieldset() {
    let id = Uuid::new_v4().to_string();
    let data = format!(r#"
    {{
        "data": {{
            "id": "{}",
            "type": "tests",
            "attributes": {{
                "body": "test",
                "title": "test",
                "published": true
            }}
        }}
    }}"#,
                       &id);

    do_post(&data);

    {
        let patch = format!(r#"
        {{
            "data": {{
                "id": "{}",
                "type": "tests",
                "attributes": {{
                    "title": "funky"
                }}
            }}
        }}"#,
                            &id);

        let updated = do_patch_with_url(&id, &patch, "fields[tests]=title");
        let expected = JsonApiData::new(Some(id),
                                        "tests",
                                        <Test as ToJson>::Attrs::new(Some("funky".to_string()),
                                                                     Some(None),
                                                                     None));

        assert_eq!(updated.data, expected);
    }
}

fn do_get<T: Display>(id: &T) -> JsonApiContainer<JsonApiData<Test>> {
    let response = request::get(&format!("http://localhost:3000/tests/{}", id),
                                Headers::new(),
                                &app_router());
    let result = response::extract_body_to_string(response.unwrap());
    serde_json::from_str(&result).unwrap()
}

fn do_post(json: &str) -> JsonApiContainer<JsonApiData<Test>> {
    do_post_with_url(json, "http://localhost:3000/tests")
}

fn do_post_with_url(json: &str, url: &str) -> JsonApiContainer<JsonApiData<Test>> {
    let content_type: Mime = "application/vnd.api+json".parse().unwrap();

    let mut headers = Headers::new();
    headers.set::<ContentType>(ContentType(content_type));

    let response = request::post(url, headers, &json, &app_router());
    let result = response::extract_body_to_string(response.unwrap());

    serde_json::from_str(&result).unwrap()
}

fn do_patch<T: Display>(id: &T, json: &str) -> JsonApiContainer<JsonApiData<Test>> {
    do_patch_with_url(id, json, "")
}

fn do_patch_with_url<T: Display>(id: &T, json: &str, query: &str) -> JsonApiContainer<JsonApiData<Test>> {
    let content_type: Mime = "application/vnd.api+json".parse().unwrap();

    let mut headers = Headers::new();
    headers.set::<ContentType>(ContentType(content_type));

    let response = request::patch(&format!("http://localhost:3000/tests/{}?{}", id, query),
                                  headers,
                                  &json,
                                  &app_router());
    let result = response::extract_body_to_string(response.unwrap());
    serde_json::from_str(&result).unwrap()
}
