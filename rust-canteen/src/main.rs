extern crate regex;

#[cfg(test)]
mod tests;

use regex::Regex;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::collections::HashMap;
use std::net::{TcpListener, ToSocketAddrs, Shutdown};
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
    Float,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Param {
    ptype:   ParamType,
    name:    String,
}

pub trait FromUri {
    fn from_uri(data: &str) -> Self;
}

impl FromUri for String {
    fn from_uri(data: &str) -> String {
        String::from(data)
    }
}

impl FromUri for i32 {
    fn from_uri(data: &str) -> i32 {
        match data.parse::<i32>() {
            Ok(v)  => v,
            Err(e) => panic!("matched integer can't be parsed: {:?}", e),
        }
    }
}

impl FromUri for f32 {
    fn from_uri(data: &str) -> f32 {
        match data.parse::<f32>() {
            Ok(v)  => v,
            Err(e) => panic!("matched float can't be parsed: {:?}", e),
        }
    }
}

#[derive(Debug)]
pub struct Request {
    method:  Method,
    path:    String,
    headers: HashMap<String, String>,
    params:  Option<HashMap<String, String>>,
    payload: Vec<u8>,
}

impl Request {
    fn new() -> Request {
        Request {
            method:  Method::NoImpl,
            path:    String::new(),
            headers: HashMap::new(),
            params:  None,
            payload: Vec::new(),
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
        let mut buf: Vec<&str> = rqstr.splitn(2, "\r\n").collect();
        let ask: Vec<&str> = buf[0].splitn(3, ' ').collect();

        self.method = match ask[1] {
            "GET"           => Method::Get,
            "PUT" | "PATCH" => Method::Put,
            "POST"          => Method::Post,
            "DELETE"        => Method::Delete,
            _               => Method::NoImpl,
        };
        self.path = String::from(ask[2]);

        loop {
            buf = buf[1].splitn(2, "\r\n").collect();

            if buf[0] == "" {
                if buf[1] == "" {
                    /* no payload */
                    break;
                }

                let tmp: String;
                buf = buf[1].splitn(2, "\r\n").collect();
                tmp = String::from(buf[1]);
                self.payload.extend(tmp.as_bytes());
                break;
            }

            let hdr: Vec<&str> = buf[0].splitn(2, ": ").collect();

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

pub trait ToOutput {
    fn to_output(&self) -> &[u8];
}

impl ToOutput for String {
    fn to_output(&self) -> &[u8] {
        self.as_bytes()
    }
}

#[derive(Debug)]
pub struct Response {
    code:       i32,
    ctype:      String,
    headers:    HashMap<String, String>,
    payload:    Vec<u8>,
}

impl Response {
    fn new() -> Response {
        Response {
            code:       200,
            ctype:      String::from("text/plain"),
            headers:    HashMap::new(),
            payload:    Vec::new(),
        }
    }

    pub fn set_code(&mut self, code: i32) {
        self.code = code;
    }

    pub fn set_content_type(&mut self, ctype: &str) {
        self.ctype = String::from(ctype);
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.insert(String::from(key), String::from(value));
    }

    pub fn append<T: ToOutput>(&mut self, payload: T) {
        self.payload.extend(payload.to_output().into_iter());
    }

    fn gen_output(&self) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::new();
        let mut inter = String::new();

        inter.push_str("HTTP/1.1 200 OK\r\n");
        inter.push_str("Connection: close\r\n");
        inter.push_str("Server: canteen/0.0.1\r\n");
        inter.push_str(&format!("Content-Type: {}\r\n", self.ctype));
        inter.push_str(&format!("Content-Length: {}\r\n", self.payload.len()));
        inter.push_str("\r\n\r\n");

        println!("{}", inter);

        output.extend(inter.as_bytes());
        output.extend(self.payload.iter());

        output
    }
}

#[derive(Debug)]
pub struct Route {
    pathdef: String,
    matcher: Regex,
    params:  HashMap<String, ParamType>,
    handler: fn(Request) -> Response,
}

impl Route {
    fn new(path: &str, handler: fn(Request) -> Response) -> Route {
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
                                "string" | "str"    => ParamType::String,
                                "integer" | "int"   => ParamType::Integer,
                                "float"             => ParamType::Float,
                                _                   => ParamType::String,

                            }
                        }
                        None        => ParamType::String,
                    };

                    let mstr: String = match ptype {
                        ParamType::String  => String::from("(?:[^/])+"),
                        ParamType::Integer => String::from("-*[0-9]+"),
                        ParamType::Float   => String::from("-*[0-9]*[.]?[0-9]+"),
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
            handler: handler,
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

    fn _no_op(req: Request) -> Response {
        let mut res = Response::new();

        res.append(String::from(req.path));

        res
    }
}

pub struct Canteen {
    routes: HashMap<String, Route>,
}


impl Canteen {
    fn new() -> Canteen {
        Canteen {
            routes: HashMap::new(),
        }
    }

    fn add_route(&mut self, path: &str, handler: fn(Request) -> Response) {
        let pc = String::from(path);

        if self.routes.contains_key(&pc) {
            panic!("a route handler for {} has already been defined!", path);
        }

        self.routes.insert(String::from(path), Route::new(&path, handler));
    }

    fn run<A: ToSocketAddrs>(&self, addr: A) {
        let listener = TcpListener::bind(addr).unwrap();

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
    }
}

fn main() {
    let mut cnt = Canteen::new();

    cnt.add_route("/hello", Route::_no_op);
    cnt.run(("127.0.0.1", 8080));
}
