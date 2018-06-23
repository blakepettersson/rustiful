#![feature(try_from)]
extern crate rustiful;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate serde_json;
extern crate uuid;

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

mod iron;
mod conversion_tests;
mod params_tests;
mod resources;
