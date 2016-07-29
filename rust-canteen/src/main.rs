extern crate mio;
extern crate regex;

#[cfg(test)]
mod tests;

mod route;
mod request;
mod response;

use std::io::Result;
use std::io::prelude::*;
use std::net::ToSocketAddrs;
use std::collections::HashMap;
use mio::tcp::{TcpListener, TcpStream};
use mio::util::Slab;
use mio::*;

use route::*;
use request::*;
use response::*;

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
            i_buf:  Vec::new(),
            o_buf:  Vec::new(),
        }
    }

    fn readable(&mut self) -> Result<bool> {
        let mut buf: Vec<u8> = Vec::with_capacity(2048);

        match self.sock.try_read_buf(&mut buf) {
            Ok(size)  => {
                match size {
                    Some(sz) => {
                        println!("read {} ({}) bytes", sz, buf.len());
                        self.events.remove(EventSet::readable());
                        self.events.insert(EventSet::writable());
                        self.i_buf.extend(buf);
                        return Ok(true);
                    },
                    None     => {
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
    fn writable(&mut self) -> Result<bool> {
        let tmp = self.o_buf.clone();
        let out = tmp.as_slice();
        match self.sock.write(&out) {
            Ok(sz)  => {
                if sz == self.o_buf.len() {
                    /* we did it! */
                    self.events.remove(EventSet::writable());
                    return Ok(true);
                } else {
                    let tmp: Vec<u8> = self.o_buf.split_off(sz);
                    println!("{:?}, -{}, {:?}", out, sz, tmp);
                    self.o_buf = tmp;

                    return Ok(false);
                }
            },
            Err(e)  => {
                panic!("failed to write to socket! <token: {:?}> <error: {:?}>", self.token, e);
            }
        }

        Ok(false)
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
    routes: HashMap<String, Route>,
    server: TcpListener,
    token:  Token,
    conns:  Slab<Client>,
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
            match self.get_client(token).writable() {
                Ok(true)    => { self.reset_connection(token); },
                Ok(false)   => { let _ = self.get_client(token).reregister(evl); },
                Err(_)      => { panic!("something really bad happened!"); },
            }
        }
    }
}

impl Canteen {
    fn new<A: ToSocketAddrs>(addr: A) -> Canteen {
        Canteen {
            routes: HashMap::new(),
            server: TcpListener::bind(&addr.to_socket_addrs().unwrap().next().unwrap()).unwrap(),
            token:  Token(1),
            conns:  Slab::new_starting_at(Token(2), 2048),
        }
    }

    fn add_route(&mut self, path: &str, mlist: Vec<Method>, handler: fn(Request) -> Response) {
        let pc = String::from(path);

        if self.routes.contains_key(&pc) {
            panic!("a route handler for {} has already been defined!", path);
        }

        self.routes.insert(String::from(path), Route::new(&path, mlist, handler));
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

    fn find_handler(&self, req: &Request) -> (fn(Request) -> Response) {
        let mut handler: fn(Request) -> Response = Route::_no_op;

        for (_, route) in &self.routes {
            match (route).is_match(req) {
                true  => { handler = route.handler; break; },
                false => continue,
            }
        }

        handler
    }

    fn readable(&mut self, evl: &mut EventLoop<Canteen>, token: Token) -> Result<bool> {
        match self.get_client(token).readable() {
            Ok(true)    => {
                let buf = self.get_client(token).i_buf.clone();
                let rqstr = String::from_utf8(buf).unwrap();
                let req = Request::from_str(&rqstr);
                let handler: fn(Request) -> Response = self.find_handler(&req);

                self.get_client(token).o_buf.extend(handler(req).gen_output());
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

    fn run(&mut self) {
        let mut evl = EventLoop::new().unwrap();
        self.register(&mut evl).ok();
        evl.run(self).unwrap();
    }
}

fn my_handler(req: Request) -> Response {
    let mut res = Response::new();

    res.set_content_type("text/html");
    res.append(String::from("<html><head><style>body { font-family: helvetica; } p { font-size: 12px; }</style></head>\
                             <body><h4>It's alive!</h4><p>Welcome to Canteen! :)</p></body></html>"));

    res
}

fn main() {
    let mut cnt = Canteen::new(("127.0.0.1", 8080));
    cnt.add_route("/hello", vec![Method::Get], my_handler);
    cnt.run();
}
