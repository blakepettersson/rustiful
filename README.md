# Introduction

Create [JSONAPI](http://jsonapi.org), um, APIs in Rust.

Rustiful is based on [Iron](http://ironframework.io) and works with stable Rust (>=1.15).    

## TODO

This is still very much a work in progress. The API _will_ change and there's quite a few features that are not 
currently implemented, including but not limited to:

- [ ] `meta` information
- [ ] `self` links
- [ ] Links in general
- [ ] Includes
- [ ] Relationships
- [ ] Filters
- [ ] Pagination
- [ ] Perhaps even [rocket](http://rocket.rs) support?

## Features implemented so far

- [x] GET/POST/PATCH/DELETE
- [x] `sort` - This means that you can access the sort parameters in a type-safe way. 
- [x] `fields` - This means that you can access the field parameters in a type-safe way.

## Installation

If you're using Cargo, add rustiful, serde and iron to your Cargo.toml. You'll probably want to have uuid support, 
which can be added using the `uuid` feature.

```toml
[dependencies]
iron = "0.5"
serde = "1.0"
serde_derive = "1.0"
rustiful = { version = "0.1", features = ["uuid", "iron"] }
rustiful-derive = { version = "0.1", features = ["uuid"] }
```

## How-to

First off, we need to have a type that we want to represent as a JSONAPI resource. To do so we need a struct that at the 
very least has an id field. The id field needs to either be named `id` or be annotated with a `JsonApiId` attribute. 

Once we have that, we can add the `JsonApi` attribute to the struct itself. This will generate the JSONAPI 
representation for the given type, as well as generating a type-safe query param type. 

You can also optionally add Serde's `Serialize` and `Deserialize` derives in case you need to rename a field and/or 
rename the resource name in the type's JSONAPI representation; if you then use Serde's `rename` attribute the generated 
JSONAPI type will use the rename attributes when serializing and deserializing.

```rust
extern crate rustiful;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate rustiful_derive;

use rustiful::*;
use rustiful::iron::*;

#[derive(Default, JsonApi)]
pub struct Todo {
    id: String,
    title: String,
    body: Option<String>,
    published: bool,
}
```

Once we have a type to use, we need a way to CRUD the resource. This is done by implementing any combination of 
`JsonGet`, `JsonPost`, `JsonIndex`, `JsonDelete` or `JsonPatch`. Each of these traits have an `Error` type and a 
`Context` type. The `Error` type needs to implement `std::error::Error`, and is used to handle any Error that might 
happen during a CRUD operation. You can use the same error type for all HTTP verb traits, or implement an error type 
per HTTP method. 

For the error type, we also need to specify what HTTP error code the error corresponds to. This is done by using the
`From` trait, where you convert a reference of your error type to a `Status` type.  

The `Context` type can be any type that implements `FromRequest`. You use this for things that needs to be initialized 
on each request (such as a database connection). `FromRequest` also requires an error type to be set. Within 
`FromRequest` you have full access to the incoming Iron request. If anything errors within `FromRequest` a 500 will be 
returned. There's currently no support for setting custom HTTP error codes for errors that happen in `FromRequest` 
impls.

See below for an example with a stub `From` implementation for `Status` and a stub `FromRequest`. We'll stub out
the resource methods with an error for now.

```rust
use rustiful::status::Status;

// `std::error::Error` implementation omitted
pub struct MyErr(String);

// Converts an error to a status code.
impl<'a> From<&'a MyErr> for Status {
    fn from(err: &'a MyErr) -> Self {
        rustiful::status::InternalServerError
    }
}

pub struct Context {}

// Initializes a `Context` from a request.
impl FromRequest for Context {
    type Error = MyErr;
    fn from_request(request: &Request) -> Result<Self, Self::Error> {
        Ok(Context {})
    }
}

impl JsonGet for Todo {
    type Error = MyErr;
    type Context = Context;

    fn find(id: String,
            _: &Self::Params,
            ctx: Self::Context)
            -> Result<Option<JsonApiData<Self::Attrs>>, Self::Error> {
        Err(MyErr("Unimplemented"))
    }
}

impl JsonIndex for Todo {
    type Error = MyErr;
    type Context = Context;

    fn find_all(params: &Self::Params, ctx: Self::Context) -> Result<Vec<JsonApiData<Self::Attrs>>, Self::Error> {
        Err(MyErr("Unimplemented"))
    }
}

impl JsonDelete for Todo {
    type Error = MyErr;
    type Context = Context;

    fn delete(id: String, ctx: Self::Context) -> Result<(), Self::Error> {
        Err(MyErr("Unimplemented"))
    }
}

impl JsonPost for Todo {
    type Error = MyErr;
    type Context = Context;

    fn create(json: JsonApiData<Self::Attrs>, 
              params: &Self::Params, 
              ctx: Self::Context) 
              -> Result<JsonApiData<Self::Attrs>, Self::Error> {
        Err(MyErr("Unimplemented"))
    }
}

impl JsonPatch for Todo {
    type Error = MyErr;
    type Context = Context;

    fn update(id: String,
              json: JsonApiData<Self::Attrs>,
              params: &Self::Params,
              ctx: Self::Context)
              -> Result<JsonApiData<Self::Attrs>, Self::Error> {
        Err(MyErr("Unimplemented"))              
    }
}
```

Finally, we need to wire the resource so that it can actually be accessed over HTTP. To do this we have a 
`JsonApiRouterBuilder` which will construct an Iron chain that is used to start the web server. 

```rust
    extern crate iron;

    fn app_router() -> iron::Chain {
        let mut router = JsonApiRouterBuilder::default();
        router.jsonapi_get::<Todo>();
        router.jsonapi_post::<Todo>();
        router.jsonapi_index::<Todo>();
        router.jsonapi_patch::<Todo>();
        router.jsonapi_delete::<Todo>();
        router.build()    
    }

    fn main() {
        Iron::new(app_router()).http("localhost:3000").unwrap()
    }
```

Once we have built the chain, we add it to the Iron constructor and start the web server. The resource path is the 
pluralized and hyphenated name of the resource type name, in lower-case. In the case of the example above that means 
that the routes are the following:
 
```
GET /todos
GET /todos/:id
POST /todos
PATCH /todos
DELETE /todos
```

If we run the example above with `cargo run` and then `curl` the server, we'll get back a JSONAPI error object.

```bash
$ curl -i http://localhost:3000/todos/
HTTP/1.1 500 Internal Server Error
Content-Type: application/vnd.api+json
Content-Length: 78
Date: Thu, 25 May 2017 15:50:25 GMT

{"errors":[{"title":"Unimplemented","status":"500","detail":"Unimplemented"}]}
```

Let's amend the `JsonIndex` impl to return a list with a single item in it.

```rust
impl JsonIndex for Todo {
    type Error = MyErr;
    type Context = Context;

    fn find_all(params: &Self::Params, ctx: Self::Context) -> Result<Vec<JsonApiData<Self::Attrs>>, Self::Error> {
        Ok(vec![Todo {
                    id: "1".to_string(),
                    body: "test".to_string(),
                    title: "test".to_string(),
                    published: true
                }.into_json(params)])
    }
}
```
If we now run curl the returned JSON should look something like this:

```bash
$ curl -i http://localhost:3000/todos/
HTTP/1.1 200 OK
Content-Type: application/vnd.api+json
Content-Length: 97
Date: Thu, 25 May 2017 16:16:20 GMT

[{"data":{"id":"1","type":"todos","attributes":{"title":"test","body":"test","published":true}}}]
```

This should be enough to get started; there's a more involved example using Diesel and connection pooling in the 
`examples` directory of this repo.
  
There's one more thing to show. You have full access to the `sort` and `fields` parameters via the params argument 
(`Self::Params`). So far this is only implemented on `JsonGet` and `JsonIndex`. 

The `Self::Params` type is an alias for `JsonApiParams<F, S>`, which has three fields: `sort`, which gives access to the 
sort query parameter, `fieldset` which gives access to the `fields` query parameter, and `query_params` which gives 
access to all other query parameters. Here's an example when using the sort parameters with Diesel 
(This assumes that you have the appropriate Diesel attributes set on `Todo`).

```rust
// Rustiful
use self::todo::sort::*;
use rustiful::SortOrder::*;
// Diesel
use self::todos as column;
use self::todos::dsl::todos as table;


impl JsonIndex for Todo {
    type Error = MyErr;
    type Context = Context;
    
    fn find_all(params: &Self::Params, ctx: Self::Context) -> Result<Vec<JsonApiData<Self::Attrs>>, Self::Error> {
        let mut query = table.into_boxed();

        {
            use self::todo::sort::*;
            use self::todos as column;
            use rustiful::SortOrder::*;

            let mut order_columns: Vec<Box<BoxableExpression<table, Pg, SqlType=()>>> = Vec::new();

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
                3 => query = query.order((order_columns.remove(0), order_columns.remove(0), order_columns.remove(0))),
                4 => query = query.order((order_columns.remove(0), order_columns.remove(0), order_columns.remove(0), order_columns.remove(0))),
                _ => return Err(MyErr("too many sort columns".to_string()))
            }
        }

        query
            .load::<Todo>(/* Add connection here */) 
            .map(|r| r.into_json(params))
            .map_err(|e| MyErr("Failed to load query"))
    }    
}
```   

If you have any questions or want to file a bug report, feel free to submit a Github issue.
