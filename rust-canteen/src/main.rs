extern crate mio;
extern crate regex;

#[cfg(test)]
mod tests;

mod route;
mod request;
mod response;

use mio::*;
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use mio::tcp::TcpListener;

/*
use regex::Regex;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::thread;
*/

use route::*;
use request::*;
use response::*;

const __SERVER: Token = Token(0);
const __CLIENT: Token = Token(1);

pub struct Canteen {
    routes: HashMap<String, Route>,
    server: TcpListener,
}

impl Handler for Canteen {
    type Timeout = ();
    type Message = u32;

    fn ready(&mut self, evl: &mut EventLoop<Canteen>, token: Token, _: EventSet) {
        match token {
            __SERVER => {
                let sock = self.server.accept();
            },
            __CLIENT => { evl.shutdown(); },
            _        => { panic!("unexpected token"); },
        }
    }

    fn notify(&mut self, _: &mut EventLoop<Canteen>, msg: u32) {
    }
}

impl Canteen {
    fn new<A: ToSocketAddrs>(addr: A) -> Canteen {
        Canteen {
            routes: HashMap::new(),
            server: TcpListener::bind(&addr.to_socket_addrs().unwrap().next().unwrap()).unwrap(),
        }
    }

    fn add_route(&mut self, path: &str, handler: fn(Request) -> Response) {
        let pc = String::from(path);

        if self.routes.contains_key(&pc) {
            panic!("a route handler for {} has already been defined!", path);
        }

        self.routes.insert(String::from(path), Route::new(&path, handler));
    }

    fn run(&mut self) {
        let mut evl = EventLoop::new().unwrap();
        evl.register(&self.server, __SERVER, EventSet::readable(), PollOpt::edge()).unwrap();
        evl.run(self).unwrap();

        /*
        for stream in listener.incoming() {
            match stream {
                Ok(stream)  => {
                    let mut rqstr = String::new();
                    let mut buf = String::new();
                    let mut reader = BufReader::new(stream);
                    let mut handler: fn(Request) -> Response = Route::_no_op;

                    while reader.read_to_string(&mut buf).unwrap() > 0 {
                        rqstr.push_str(&buf);
                    }

                    println!("{}", rqstr);
                    let req = Request::from_str(&rqstr);

                    for (_, route) in &self.routes {
                        match route.is_match(&req.path) {
                            true  => { handler = route.handler; break; },
                            false => continue,
                        }
                    }

                    //thread::spawn(move || {
                        let stream = reader.into_inner();
                        let mut writer = BufWriter::new(stream);
                        let res = handler(req);
                        let out = res.gen_output();
                        println!("sending: '{:?}'", out);

                        {
                            writer.write(&out.as_slice()).unwrap();
                        }

                        let mut stream = writer.into_inner().unwrap();
                        let _ = stream.shutdown(Shutdown::Both);
                    //});
                },
                Err(e)      => { println!("{}", e); },
            }
        }

        drop(listener);
        */
    }
}

fn main() {
    let mut cnt = Canteen::new(("127.0.0.1", 8080));
    cnt.add_route("/hello", Route::_no_op);
    cnt.run();
}
