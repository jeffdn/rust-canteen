// Copyright (c) 2016
// Jeff Nettleton
//
// Licensed under the MIT license (http://opensource.org/licenses/MIT). This
// file may not be copied, modified, or distributed except according to those
// terms

#[cfg(feature = "pool-postgres")]
pub mod pool_postgres;
#[cfg(feature = "pool-postgres")]
pub use pool_postgres::*;

// Here and down resides the default implementation
#[cfg(not(any(feature = "pool-postgres")))]
use request::*;

#[cfg(not(any(feature = "pool-postgres")))]
pub struct Context {
    pub request: Request
}

#[cfg(not(any(feature = "pool-postgres")))]
impl Context {
    pub fn new(req: Request) -> Context {
        Context {
            request: req
        }
    }
}

#[cfg(not(any(feature = "pool-postgres")))]
pub struct ConnectionPool {
}

#[cfg(not(any(feature = "pool-postgres")))]
impl ConnectionPool {
}
