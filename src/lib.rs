/* Copyright (c) 2016
 * Jeff Nettleton
 *
 * Licensed under the MIT license (http://opensource.org/licenses/MIT). This
 * file may not be copied, modified, or distributed except according to those
 * terms
 */

extern crate mio;
extern crate regex;

#[cfg(test)]
mod tests;

pub mod utils;
pub mod route;
pub mod request;
pub mod response;

use std::io::Result;
use std::io::prelude::*;
use std::net::ToSocketAddrs;
use std::collections::HashMap;
use std::collections::HashSet;
use mio::tcp::{TcpListener, TcpStream};
use mio::util::Slab;
use mio::*;

pub use utils::*;
pub use route::*;
pub use request::*;
pub use response::*;

struct Client {
    sock:   TcpStream,
    token:  Token,
    events: EventSet,
    i_buf:  Vec<u8>,
    o_buf:  Vec<u8>,
}

impl Client {
    fn new(sock: TcpStream, token: Token) -> Client {
        Client {
            sock:   sock,
            token:  token,
            events: EventSet::hup(),
            i_buf:  Vec::with_capacity(2048),
            o_buf:  Vec::with_capacity(2048),
        }
    }

    fn receive(&mut self) -> Result<bool> {
        let mut buf: Vec<u8> = Vec::with_capacity(2048);

        match self.sock.try_read_buf(&mut buf) {
            Ok(size)  => {
                match size {
                    Some(_) => {
                        self.events.remove(EventSet::readable());
                        self.events.insert(EventSet::writable());
                        self.i_buf.extend(buf);
                        return Ok(true);
                    },
                    None    => {
                        return Ok(false);
                    },
                }
            },
            Err(e)  => {
                panic!("failed to read from socket! <token: {:?}> <error: {:?}>", self.token, e);
            },
        }

        Ok(false)
    }

    /* write the client's output buffer to the socket.
     *
     * the following return values mean:
     *  - Ok(true):  we can close the connection
     *  - Ok(false): keep listening for writeable event and continue next time
     *  - Err(e):    something dun fucked up
     */
    fn send(&mut self) -> Result<bool> {
        while self.o_buf.len() > 0 {
            match self.sock.write(&self.o_buf.as_slice()) {
                Ok(sz)  => {
                    if sz == self.o_buf.len() {
                        /* we did it! */
                        self.events.remove(EventSet::writable());
                        break;
                    } else {
                        /* keep going */
                        self.o_buf = self.o_buf.split_off(sz);
                    }
                },
                Err(e)  => {
                    panic!("failed to write to socket! <token: {:?}> <error: {:?}>", self.token, e);
                }
            }
        }

        Ok(true)
    }

    fn register(&mut self, evl: &mut EventLoop<Canteen>) -> Result<()> {
        self.events.insert(EventSet::readable());

        evl.register(&self.sock, self.token, self.events, PollOpt::edge() | PollOpt::oneshot())
           .or_else(|e| {
               panic!("failed to register client! <token: {:?}> <error: {:?}>", self.token, e);
           })
    }

    fn reregister(&mut self, evl: &mut EventLoop<Canteen>) -> Result<()> {
        evl.reregister(&self.sock, self.token, self.events, PollOpt::edge() | PollOpt::oneshot())
           .or_else(|e| {
               panic!("failed to re-register client! <token: {:?}> <error: {:?}>", self.token, e);
           })
    }
}

/* our primary object. similar interface to Flask, the
 * Python microframework. much faster, however! :)
 */
pub struct Canteen {
    routes:  HashMap<RouteDef, Route>,
    rcache:  HashMap<RouteDef, RouteDef>,
    server:  TcpListener,
    token:   Token,
    conns:   Slab<Client>,
    default: fn(&Request) -> Response,
}

impl Handler for Canteen {
    type Timeout = ();
    type Message = u32;

    fn ready(&mut self, evl: &mut EventLoop<Canteen>, token: Token, events: EventSet) {
        if events.is_error() || events.is_hup() {
            self.reset_connection(token);
            return;
        }

        if events.is_readable() {
            if self.token == token {
                self.accept(evl);
            } else {
                self.readable(evl, token)
                    .and_then(|_| self.get_client(token)
                                      .reregister(evl))
                    .unwrap_or_else(|e| {
                        panic!("read event failed! <token: {:?}> <error: {:?}>", token, e);
                    });
            }

            return;
        }

        if events.is_writable() {
            match self.get_client(token).send() {
                Ok(true)    => { self.reset_connection(token); },
                Ok(false)   => { let _ = self.get_client(token).reregister(evl); },
                Err(_)      => { panic!("something really bad happened!"); },
            }
        }
    }
}

impl Canteen {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Canteen {
        Canteen {
            routes:  HashMap::new(),
            rcache:  HashMap::new(),
            server:  TcpListener::bind(&addr.to_socket_addrs().unwrap().next().unwrap()).unwrap(),
            token:   Token(1),
            conns:   Slab::new_starting_at(Token(2), 2048),
            default: Route::err_404,
        }
    }

    pub fn add_route(&mut self, path: &str, mlist: &[Method],
                     handler: fn(&Request) -> Response) -> &mut Canteen {
        let mut methods: HashSet<Method> = HashSet::new();

        /* make them unique */
        for m in mlist {
            methods.insert(*m);
        }

        for m in methods {
            let rd = RouteDef {
                pathdef:    String::from(path),
                method:     m,
            };

            if self.routes.contains_key(&rd) {
                panic!("a route handler for {} has already been defined!", path);
            }

            self.routes.insert(rd, Route::new(&path, m, handler));
        }

        self
    }

    pub fn set_default(&mut self, handler: fn(&Request) -> Response) {
        self.default = handler;
    }

    fn get_client<'a>(&'a mut self, token: Token) -> &'a mut Client {
        &mut self.conns[token]
    }

    fn accept(&mut self, evl: &mut EventLoop<Canteen>) {
        let (sock, _) = match self.server.accept() {
            Ok(s)   => {
                match s {
                    Some(sock)  => sock,
                    None        => {
                        panic!("failed to accept new connection!");
                    }
                }
            },
            Err(e)  => {
                panic!("failed to accept new connection! <error: {:?}>", e);
            },
        };

        match self.conns.insert_with(|token| Client::new(sock, token)) {
            Some(token) => {
                match self.get_client(token).register(evl) {
                    Ok(_)   => {},
                    Err(e)  => {
                        panic!("failed to register client! <token: {:?}> <error: {:?}>", token, e);
                    },
                }
            },
            None        => {
                panic!("failed to add client connection!");
            },
        }

        self.reregister(evl);
    }

    fn handle_request(&mut self, req: &mut Request) -> Vec<u8> {
        let resolved = RouteDef { pathdef: req.path.clone(), method: req.method };
        let mut handler: fn(&Request) -> Response = self.default;

        if self.rcache.contains_key(&resolved) {
            let route = self.routes.get(self.rcache.get(&resolved).unwrap()).unwrap();

            handler = route.handler;
            req.params = route.parse(&req.path);
        } else {
            for (path, route) in &self.routes {
                match (route).is_match(req) {
                    true  => {
                        handler = route.handler;
                        req.params = route.parse(&req.path);
                        self.rcache.insert(resolved, (*path).clone());
                        break;
                    },
                    false => continue,
                }
            }
        }

        handler(req).gen_output()
    }

    fn readable(&mut self, evl: &mut EventLoop<Canteen>, token: Token) -> Result<bool> {
        match self.get_client(token).receive() {
            Ok(true)    => {
                let buf = self.get_client(token).i_buf.clone();
                let rqstr = String::from_utf8(buf).unwrap();
                let mut req = Request::from_str(&rqstr);
                let output = self.handle_request(&mut req);

                self.get_client(token).o_buf.extend(output);
            },
            Ok(false)   => {},
            Err(e)      => {
                panic!("message wasn't actually readable! <error: {:?}>", e);
            },
        };

        let _ = self.get_client(token).reregister(evl);

        Ok(true)
    }

    fn reset_connection(&mut self, token: Token) {
        /* kill the connection */
        self.conns.remove(token);
    }

    fn register(&mut self, evl: &mut EventLoop<Canteen>) -> Result<()> {
        evl.register(&self.server, self.token, EventSet::readable(), PollOpt::edge() | PollOpt::oneshot())
           .or_else(|e| {
               panic!("failed to register server! <token: {:?}> <error: {:?}>", self.token, e);
           })
    }

    fn reregister(&mut self, evl: &mut EventLoop<Canteen>) {
        match evl.reregister(&self.server, self.token,
                             EventSet::readable(),
                             PollOpt::edge() | PollOpt::oneshot()) {
            Ok(_)   => {},
            Err(e)  => {
               panic!("failed to register server! <token: {:?}> <error: {:?}>", self.token, e);
           }
        };
    }

    pub fn run(&mut self) {
        let mut evl = EventLoop::new().unwrap();
        self.register(&mut evl).ok();
        evl.run(self).unwrap();
    }
}
