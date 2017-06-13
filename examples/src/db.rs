extern crate r2d2;
extern crate r2d2_diesel;
extern crate iron;

use self::iron::prelude::*;
use self::r2d2::{GetTimeout, Pool, PooledConnection};

use self::r2d2_diesel::ConnectionManager;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use rustiful::iron::FromRequest;
use std::env;

/// This struct is a simple wrapper for a Postgres connection pool.
pub struct DB(PooledConnection<ConnectionManager<PgConnection>>);

impl DB {
    pub fn conn(&self) -> &PgConnection {
        &*self.0
    }
}

pub fn create_db_pool() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();

    let database_url = env::var("POSTGRES_URL").expect("POSTGRES_URL must be set");
    let config = r2d2::Config::default();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::new(config, manager).expect("Failed to create pool.")
}

lazy_static! {
    pub static ref DB_POOL: Pool<ConnectionManager<PgConnection>> = create_db_pool();
}

/// This fetches a connection from the connection pool on each request. If the attempt to retrieve
/// a connection fails for whatever reason, a 500 Internal Server Error will be sent to the
/// client.
impl FromRequest for DB {
    type Error = GetTimeout;

    fn from_request(_: &Request) -> Result<DB, Self::Error> {
        match DB_POOL.get() {
            Ok(conn) => Ok(DB(conn)),
            Err(e) => Err(e),
        }
    }
}
