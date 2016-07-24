extern crate regex;

#[cfg(test)]
mod tests;

use regex::Regex;
use std::io::Read;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::thread;

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub enum Method {
    Get,
    Put,
    Post,
    Delete,
    NoImpl,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum ParamType {
    Integer,
    String,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Param {
    ptype:   ParamType,
    name:    String,
}

pub trait FromUri: Sized {
    fn from_uri(data: &str) -> Self;
}

impl FromUri for String {
    fn from_uri(data: &str) -> String {
        String::from(data)
    }
}

impl FromUri for i32 {
    fn from_uri(data: &str) -> i32 {
        data.parse::<i32>().unwrap()
    }
}

#[derive(Debug)]
pub struct Request {
    method:  Method,
    path:    String,
    headers: HashMap<String, String>,
    params:  Option<HashMap<String, String>>,
    payload: String,
}

impl Request {
    fn new() -> Request {
        Request {
            method:  Method::NoImpl,
            path:    String::new(),
            headers: HashMap::new(),
            params:  None,
            payload: String::new(),
        }
    }

    fn _get(&self, name: &str) -> Option<&String> {
        let val: Option<&String> = match self.params {
            Some(ref p) => p.get(name),
            None        => None,
        };

        val
    }

    pub fn get<T>(&self, name: &str) -> T where T: FromUri {
        match self._get(&name) {
            Some(item) => FromUri::from_uri(&item),
            None       => panic!("invalid route parameter {:?}", name),
        }
    }

    pub fn from_str(rqstr: &str) -> Request {
        let mut req = Request::new();
        req.parse(rqstr);
        req
    }

    pub fn parse(&mut self, rqstr: &str) {
        let mut chunks: Vec<&str> = rqstr.splitn(2, "\r\n\r\n").collect();
        let mut header: Vec<&str> = chunks.pop().unwrap().split("\r\n").collect();
        let ask: Vec<&str> = header.pop().unwrap().splitn(3, ' ').collect();

        self.method = match ask[0] {
            "GET"           => Method::Get,
            "PUT" | "PATCH" => Method::Put,
            "POST"          => Method::Post,
            "DELETE"        => Method::Delete,
            _               => Method::NoImpl,
        };
        self.path = String::from(ask[1]);
        self.payload = match chunks.pop() {
            Some(x) => String::from(x),
            None    => String::new(),
        };

        for line in header {
            let hdr: Vec<&str> = line.splitn(2, ": ").collect();

            if hdr.len() == 2 {
                self.headers.insert(String::from(hdr[0]), String::from(hdr[1]));
            }
        }
    }

    pub fn has_params(&self) -> bool {
        match self.params {
            Some(ref p) => true,
            None        => false,
        }
    }
}

#[derive(Debug)]
pub struct Route {
    pathdef: String,
    matcher: Regex,
    params:  HashMap<String, ParamType>,
    handler: Option<fn(Request) -> String>,
}

impl Route {
    fn new(path: &str) -> Route {
        let re = Regex::new(r"^<(?:(int|str):)?([\w_][a-zA-Z0-9_]*)>$").unwrap();
        let parts: Vec<&str> = path.split('/').filter(|&s| s != "").collect();
        let mut matcher: String = String::from(r"^");
        let mut params: HashMap<String, ParamType> = HashMap::new();

        for part in parts {
            let chunk: String = match re.is_match(part) {
                true  => {
                    let mut rc = String::new();
                    let caps = re.captures(part).unwrap();
                    let param = caps.at(2).unwrap().clone();
                    let ptype: ParamType = match caps.at(1) {
                        Some(x)     => {
                            match x.as_ref() {
                                "str" => ParamType::String,
                                "int" => ParamType::Integer,
                                _     => ParamType::String,

                            }
                        }
                        None        => ParamType::String,
                    };

                    let mstr: String = match ptype {
                        ParamType::String  => String::from("(?:[^/])+"),
                        ParamType::Integer => String::from("[0-9]+"),
                    };

                    rc.push_str("/(?P<");
                    rc.push_str(&param);
                    rc.push_str(">");
                    rc.push_str(&mstr);
                    rc.push_str(")");
                    params.insert(String::from(param), ptype);

                    rc
                },
                false => String::from("/") + part,
            };

            matcher.push_str(&chunk);
        }

        /* end the regex with an optional final slash and a string terminator */
        matcher.push_str("/?$");

        Route {
            pathdef: String::from(path),
            matcher: Regex::new(&matcher).unwrap(),
            params:  params,
            handler: None,
        }
    }

    fn is_match(&self, path: &str) -> bool {
        self.matcher.is_match(&path)
    }

    fn parse(&self, path: &str) -> Option<HashMap<String, String>> {
        let mut params: HashMap<String, String> = HashMap::new();

        return match self.matcher.is_match(&path) {
            true  => {
                let caps = self.matcher.captures(path).unwrap();
                for (param, _) in &self.params {
                    params.insert(param.clone(), String::from(caps.name(&param).unwrap()));
                }

                Some(params)
            },
            false => None,
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut rqstr: String = String::new();
    let mut buf: String = String::new();

    while stream.read_to_string(&mut buf).unwrap() > 0 {
        rqstr.push_str(&buf);
    }

    let req = Request::from_str(&rqstr);
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream)  => {
                thread::spawn(move || {
                    handle_client(stream)
                });
            },
            Err(e)      => { println!("{}", e); },
        }
    }

    drop(listener);
}
