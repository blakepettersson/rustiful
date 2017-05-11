use db::DB;
use diesel;

use diesel::*;
use errors::MyErr;
use rustiful::*;

extern crate uuid;

use self::todos::dsl::todos as table;
use self::uuid::Uuid;

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
            _: &Self::Params,
            ctx: Self::Context)
            -> Result<Option<Self>, Self::Error> {
        table.find(id).first(ctx.conn()).optional().map_err(|e| MyErr::Diesel(e))
    }
}

impl JsonPatch for Todo {
    type Error = MyErr;
    type Context = DB;

    /// Updates a record in the database. First we fetch the record from the database, then convert
    /// the record along with the JSON patch to a new instance that has the updated columns, before
    /// saving it in the database.
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

impl JsonDelete for Todo {
    type Error = MyErr;
    type Context = DB;

    /// Deletes a record from the database with the given id
    fn delete(id: Self::JsonApiIdType, ctx: Self::Context) -> Result<(), Self::Error> {
        diesel::delete(table.find(id)).execute(ctx.conn()).map(|_| ()).map_err(|e| MyErr::Diesel(e))
    }
}

impl JsonPost for Todo {
    type Error = MyErr;
    type Context = DB;

    /// Creates a record in the database. If the client specifies an id in the JSON document you
    /// must create a record with the given id. If the id is not specified (i.e the record will get
    /// an auto-generated id), then make sure that you return a record with the generated id. This
    /// is handled with the `get_result` method below.
    fn create(record: Self::Resource, ctx: Self::Context) -> Result<Self, Self::Error> {
        let todo: Todo = record.try_into().map_err(|e| MyErr::UpdateError(e))?;
        let result: NewTodo = todo.into();
        diesel::insert(&result)
            .into(table)
            .get_result::<Todo>(ctx.conn())
            .map_err(|e| MyErr::Diesel(e))
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
