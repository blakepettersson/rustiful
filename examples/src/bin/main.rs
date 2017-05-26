extern crate rustiful;
extern crate iron;
extern crate rustiful_examples;

use iron::prelude::*;
use rustiful::iron::JsonApiRouterBuilder;

use rustiful_examples::todo::*;

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
    let _server = Iron::new(app_router()).http("localhost:3000").unwrap();
}
