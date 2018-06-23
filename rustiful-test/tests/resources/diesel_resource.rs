infer_schema!("dotenv:DATABASE_URL");

use self::tests as column;
use self::tests::dsl::tests as table;
use diesel;
use diesel::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use r2d2::{Config, Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;
use rustiful::*;
use rustiful::SortOrder::*;
use std::convert::TryInto;
use std::env;
use std::error::Error;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonApi, Queryable,
         Insertable, AsChangeset)]
#[table_name = "tests"]
#[changeset_options(treat_none_as_null = "true")]
pub struct Test {
    pub id: String,
    pub title: String,
    pub body: Option<String>,
    pub published: bool
}

#[derive(Debug)]
pub enum MyErr {
    Diesel(diesel::result::Error),
    UpdateError(String)
}

impl Error for MyErr {
    fn description(&self) -> &str {
        match *self {
            MyErr::Diesel(ref err) => err.description(),
            MyErr::UpdateError(ref err) => err
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            MyErr::Diesel(ref err) => err.cause(),
            MyErr::UpdateError(_) => None
        }
    }
}

impl Display for MyErr {
    fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result {
        match *self {
            MyErr::Diesel(ref err) => err.fmt(f),
            MyErr::UpdateError(ref err) => err.fmt(f)
        }
    }
}

impl JsonGet for Test {
    type Error = MyErr;
    type Context = DB;

    fn find(
        id: Self::JsonApiIdType,
        params: &Self::Params,
        ctx: Self::Context
    ) -> Result<Option<JsonApiData<Self>>, (Self::Error, Self::Status)> {
        if id == "fail" {
            return Err(MyErr::UpdateError("test fail".to_string()).into());
        }
        table
            .find(id)
            .first::<Test>(ctx.conn())
            .optional()
            .map(|r| r.map(|i| i.into_json(params)))
            .map_err(|e| MyErr::Diesel(e).into())
    }
}

impl JsonPatch for Test {
    type Error = MyErr;
    type Context = DB;

    fn update(
        id: Self::JsonApiIdType,
        json: JsonApiData<Self>,
        params: &Self::Params,
        ctx: Self::Context
    ) -> Result<JsonApiData<Self>, (Self::Error, Self::Status)> {
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
            .map(|_| patch.into_json(params))
            .map_err(|e| MyErr::Diesel(e).into())
    }
}

impl JsonIndex for Test {
    type Error = MyErr;
    type Context = DB;

    fn find_all(
        params: &Self::Params,
        ctx: Self::Context
    ) -> Result<Vec<JsonApiData<Self>>, (Self::Error, Self::Status)> {
        let mut query = table.into_boxed();

        {
            use self::test::sort::*;
            for order in &params.sort {
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
            .map_err(|e| MyErr::Diesel(e).into())
    }
}

impl JsonDelete for Test {
    type Error = MyErr;
    type Context = DB;

    fn delete(
        id: Self::JsonApiIdType,
        ctx: Self::Context
    ) -> Result<(), (Self::Error, Self::Status)> {
        diesel::delete(table.find(id))
            .execute(ctx.conn())
            .map(|_| ())
            .map_err(|e| MyErr::Diesel(e).into())
    }
}

impl JsonPost for Test {
    type Error = MyErr;
    type Context = DB;

    fn create(
        json: JsonApiData<Self>,
        params: &Self::Params,
        ctx: Self::Context
    ) -> Result<JsonApiData<Self>, (Self::Error, Self::Status)> {
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
            .map_err(|e| MyErr::Diesel(e).into())
            .map(|_| result.into_json(params))
    }
}


lazy_static! {
    pub static ref DB_POOL: Pool<ConnectionManager<SqliteConnection>> = create_db_pool();
}

pub struct DB(pub PooledConnection<ConnectionManager<SqliteConnection>>);

impl DB {
    pub fn conn(&self) -> &SqliteConnection {
        &*self.0
    }
}

pub fn create_db_pool() -> Pool<ConnectionManager<SqliteConnection>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let config = Config::default();
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::new(config, manager).expect("Failed to create pool.")
}
