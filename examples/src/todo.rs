use db::DB;
use diesel;

use diesel::*;
use errors::MyErr;
use rustiful::*;
use rustiful::iron::status::Status;

extern crate uuid;

use self::todos::dsl::todos as table;
use self::uuid::Uuid;
use diesel::pg::Pg;

infer_schema!("dotenv:POSTGRES_URL");

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonApi, Queryable,
Insertable, AsChangeset)]
#[table_name="todos"]
#[changeset_options(treat_none_as_null = "true")]
pub struct Todo {
    id: Uuid,
    title: String,
    body: Option<String>,
    published: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Insertable)]
#[table_name="todos"]
pub struct NewTodo {
    title: String,
    body: Option<String>,
    published: bool,
}

impl JsonGet for Todo {
    type Error = MyErr;
    type Context = DB;

    /// Gets a record from the database with the given id
    fn find(id: Self::JsonApiIdType,
            params: &Self::Params,
            ctx: Self::Context)
            -> Result<Option<JsonApiData<Self>>, (Self::Error, Status)> {
        table
            .find(id)
            .first::<Todo>(ctx.conn())
            .map(|r| r.into_json(params))
            .optional()
            .map_err(|e| (MyErr::Diesel(e), Status::InternalServerError))
    }
}

impl JsonIndex for Todo {
    type Error = MyErr;
    type Context = DB;

    /// Gets all records from the database
    fn find_all(params: &Self::Params,
                ctx: Self::Context)
                -> Result<Vec<JsonApiData<Self>>, (Self::Error, Status)> {

        let mut query = table.into_boxed();

        {
            use self::todo::sort::*;
            use self::todos as column;
            use rustiful::SortOrder::*;

            let mut order_columns: Vec<Box<BoxableExpression<table, Pg, SqlType = ()>>> =
                Vec::new();

            for order in &params.sort.fields {
                match *order {
                    title(Asc) => {
                        order_columns.push(Box::new(column::title.asc()));
                    }
                    title(Desc) => {
                        order_columns.push(Box::new(column::title.desc()));
                    }
                    body(Asc) => {
                        order_columns.push(Box::new(column::body.asc()));
                    }
                    body(Desc) => {
                        order_columns.push(Box::new(column::body.desc()));
                    }
                    published(Asc) => {
                        order_columns.push(Box::new(column::published.asc()));
                    }
                    published(Desc) => {
                        order_columns.push(Box::new(column::published.desc()));
                    }
                };
            }

            // TODO: Hopefully there's a nicer way to get multiple ORDER BY clauses in this query.
            match order_columns.len() {
                1 => query = query.order(order_columns.remove(0)),
                2 => query = query.order((order_columns.remove(0), order_columns.remove(0))),
                3 => {
                    query = query.order((order_columns.remove(0),
                                         order_columns.remove(0),
                                         order_columns.remove(0)))
                }
                4 => {
                    query = query.order((order_columns.remove(0),
                                         order_columns.remove(0),
                                         order_columns.remove(0),
                                         order_columns.remove(0)))
                }
                _ => return Err((MyErr::TooManySortColumns("too many sort columns".to_string()), Status::BadRequest)),
            }
        }

        query
            .load::<Todo>(ctx.conn())
            .map(|r| r.into_json(params))
            .map_err(|e| (MyErr::Diesel(e), Status::InternalServerError))
    }
}

impl JsonPatch for Todo {
    type Error = MyErr;
    type Context = DB;

    /// Updates a record in the database. First we fetch the record from the database, then convert
    /// the record along with the JSON patch to a new instance that has the updated columns, before
    /// saving it in the database.
    fn update(id: Self::JsonApiIdType,
              json: JsonApiData<Self>,
              params: &Self::Params,
              ctx: Self::Context)
              -> Result<JsonApiData<Self>, (Self::Error, Status)> {
        let record = table
            .find(&id)
            .first(ctx.conn())
            .map_err(|e| (MyErr::Diesel(e), Status::InternalServerError))?;
        let patch = (record, json)
            .try_into()
            .map_err(|e| (MyErr::UpdateError(e), Status::ImATeapot))?;
        diesel::update(table.find(&id))
            .set(&patch)
            .execute(ctx.conn())
            .map_err(|e| (MyErr::Diesel(e), Status::InternalServerError))?;
        Ok(patch.into_json(params))
    }
}

impl JsonDelete for Todo {
    type Error = MyErr;
    type Context = DB;

    /// Deletes a record from the database with the given id
    fn delete(id: Self::JsonApiIdType, ctx: Self::Context) -> Result<(), (Self::Error, Status)> {
        diesel::delete(table.find(id))
            .execute(ctx.conn())
            .map(|_| ())
            .map_err(|e| (MyErr::Diesel(e), Status::InternalServerError))
    }
}

impl JsonPost for Todo {
    type Error = MyErr;
    type Context = DB;

    /// Creates a record in the database. If the client specifies an id in the JSON document you
    /// must create a record with the given id. If the id is not specified (i.e the record will get
    /// an auto-generated id), then make sure that you return a record with the generated id. This
    /// is handled with the `get_result` method below.
    fn create(record: JsonApiData<Self>,
              params: &Self::Params,
              ctx: Self::Context)
              -> Result<JsonApiData<Self>, (Self::Error, Status)> {
        let todo: Todo = record.try_into().map_err(|e| (MyErr::UpdateError(e), Status::ImATeapot))?;
        let result: NewTodo = todo.into();
        diesel::insert(&result)
            .into(table)
            .get_result::<Todo>(ctx.conn())
            .map(|r| r.into_json(params))
            .map_err(|e| (MyErr::Diesel(e), Status::InternalServerError))
    }
}

/// Converts a `Todo` to a `NewTodo`.
impl From<Todo> for NewTodo {
    fn from(todo: Todo) -> Self {
        NewTodo {
            title: todo.title,
            body: todo.body,
            published: todo.published,
        }
    }
}
