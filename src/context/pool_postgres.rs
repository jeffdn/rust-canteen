// Copyright (c) 2016
// Jeff Nettleton
//
// Licensed under the MIT license (http://opensource.org/licenses/MIT). This
// file may not be copied, modified, or distributed except according to those
// terms

extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;

use request::*;

use self::r2d2_postgres::{SslMode, PostgresConnectionManager, Error};

pub fn generate_pool(uri: &str) -> r2d2::Pool<PostgresConnectionManager> {
    let config: r2d2::Config<postgres::Connection, Error> = r2d2::Config::builder().pool_size(1).build();
    let manager = PostgresConnectionManager::new(uri, SslMode::None).unwrap();

    r2d2::Pool::new(config, manager).unwrap()
}

#[cfg(feature = "pool-postgres")]
pub struct Context {
    pub request: Request
}

#[cfg(feature = "pool-postgres")]
impl Context {
    pub fn new(req: Request) -> Context {
        Context {
            request: req
        }
    }
}

#[cfg(feature = "pool-postgres")]
pub struct ConnectionPool {
}

#[cfg(feature = "pool-postgres")]
impl ConnectionPool {
}
