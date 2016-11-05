// Copyright (c) 2016
// Jeff Nettleton
//
// Licensed under the MIT license (http://opensource.org/licenses/MIT). This
// file may not be copied, modified, or distributed except according to those
// terms

use std::collections::HashMap;
use rustc_serialize::{json, Decodable};

/// This enum represents the various types of HTTP requests.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Method {
    Get,
    Put,
    Post,
    Delete,
    NoImpl,
}

/// A trait that allows for extracting variables from URIs.
pub trait FromUri {
    /// A function to parse a string into the correct type.
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

impl FromUri for u32 {
    fn from_uri(data: &str) -> u32 {
        match data.parse::<u32>() {
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

/// This struct represents a request from an HTTP client.
#[derive(Debug)]
pub struct Request {
    pub method:  Method,
    pub path:    String,
    pub payload: Vec<u8>,
    pub params:  HashMap<String, String>,
    headers:     HashMap<String, String>,
}

impl Request {
    /// Create a new, empty Request.
    pub fn new() -> Request {
        Request {
            method:  Method::NoImpl,
            path:    String::new(),
            headers: HashMap::new(),
            params:  HashMap::new(),
            payload: Vec::with_capacity(2048),
        }
    }

    /// Create a Request from an HTTP request string.
    pub fn from_str(rqstr: &str) -> Request {
        let mut req = Request::new();
        req.parse(rqstr);
        req
    }

    /// Get an HTTP header contained in the Request.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use canteen::{Request, Response};
    /// use canteen::utils;
    ///
    /// // Given the route "/hello"
    /// fn handler(req: &Request) -> Response {
    ///     let browser = req.get_header("User-Agent");
    ///
    ///     match browser {
    ///         Some(ua) => utils::make_response(format!("You're using {}!", ua), "text/plain", 200),
    ///         None     => utils::make_response("Bad browser, no user agent!", "text/plain", 200),
    ///     }
    /// }
    /// ```
    pub fn get_header(&self, name: &str) -> Option<String> {
        let key = String::from(name);

        match self.headers.get(&key) {
            Some(val)   => Some(val.clone()),
            None        => None,
        }
    }

    /// Get a variable from the URI.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use canteen::{Request, Response};
    /// use canteen::utils;
    ///
    /// // Given the route "/hello/<str:name>"
    /// fn handler(req: &Request) -> Response {
    ///     let name: String = req.get("name");
    ///     utils::make_response(format!("<b>Hello, {}!</b>", name), "text/html", 200)
    /// }
    /// ```
    pub fn get<T: FromUri>(&self, name: &str) -> T {
        if !self.params.contains_key(name) {
            panic!("invalid route parameter {:?}", name);
        }

        FromUri::from_uri(&self.params[name])
    }

    /// Get a raw JSON payload from the request.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use canteen::{Request, Response};
    /// use canteen::utils;
    ///
    /// // Given the POST route "/hello"
    /// fn handler(req: &Request) -> Response {
    ///     let data = req.get_json();
    ///
    ///     match data {
    ///         Some(val) => utils::make_response(format!("We got: {}", val), "text/plain", 200),
    ///         None      => utils::make_response("We got nothing :(", "text/plain", 200),
    ///     }
    /// }
    /// ```
    pub fn get_json(&self) -> Option<json::Json> {
        match String::from_utf8(self.payload.clone()) {
            Err(_)      => None,
            Ok(payload) => {
                match json::Json::from_str(&payload) {
                    Ok(data)    => Some(data),
                    Err(_)      => None,
                }
            }
        }
    }

    /// Get a composed JSON payload from the request.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use canteen::{Request, Response};
    ///
    /// #[derive(RustcDecodable)]
    /// struct Foo {
    ///     item: i32,
    /// }
    ///
    /// // Given the POST route "/hello"
    /// fn handler(req: &Request) -> Response {
    ///     let data: Foo = req.get_json_obj();
    ///
    ///     match data {
    ///         Ok(foo) => utils::make_response(format!("We got: {}!", data.item), "text/plain", 200),
    ///         Err(_)  => utils::make_response("We got nothing :(", "text/plain", 200),
    ///     }
    /// }
    /// ```
    pub fn get_json_obj<T: Decodable>(&self) -> Result<T, json::DecoderError> {
        let data = String::from_utf8(self.payload.clone()).unwrap();
        json::decode(&data)
    }

    fn parse(&mut self, rqstr: &str) {
        let mut buf: Vec<&str> = rqstr.splitn(2, "\r\n").collect();
        let ask: Vec<&str> = buf[0].splitn(3, ' ').collect();

        self.method = match ask[0] {
            "GET"           => Method::Get,
            "PUT" | "PATCH" => Method::Put,
            "POST"          => Method::Post,
            "DELETE"        => Method::Delete,
            _               => Method::NoImpl,
        };
        self.path = String::from(ask[1]);

        loop {
            buf = buf[1].splitn(2, "\r\n").collect();

            if buf[0] == "" {
                if buf.len() == 1 || buf[1] == "" {
                    // no payload
                    break;
                }

                self.payload.extend(buf[1].as_bytes());
                break;
            }

            let hdr: Vec<&str> = buf[0].splitn(2, ": ").collect();

            if hdr.len() == 2 {
                self.headers.insert(String::from(hdr[0]), String::from(hdr[1]));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(RustcDecodable)]
    struct Foo {
        item: i32,
    }

    #[test]
    fn test_fromuri_trait_i32() {
        let pos = String::from("1234");
        assert_eq!(1234, FromUri::from_uri(&pos));

        let neg = String::from("-4321");
        assert_eq!(-4321, FromUri::from_uri(&neg));
    }

    #[test]
    fn test_fromuri_trait_u32() {
        let orig = String::from("1234");
        assert_eq!(1234, FromUri::from_uri(&orig));
    }

    #[test]
    fn test_fromuri_trait_string() {
        let orig = String::from("foobar");
        assert_eq!("foobar", <String as FromUri>::from_uri(&orig));
    }

    #[test]
    fn test_fromuri_trait_float() {
        let pos = String::from("123.45");
        assert_eq!(123.45f32, FromUri::from_uri(&pos));

        let neg = String::from("-54.321");
        assert_eq!(-54.321f32, FromUri::from_uri(&neg));
    }

    #[test]
    fn test_get_fromuri_i32() {
        let mut req = Request::new();
        req.params.insert(String::from("test"), String::from("1234"));

        assert_eq!(1234, req.get("test"));
    }

    #[test]
    fn test_get_json() {
        let mut req = Request::new();
        req.payload.extend_from_slice("{ \"item\": 123 }".as_bytes());

        let data = req.get_json().unwrap();

        assert_eq!(true, data.is_object());

        let obj = data.as_object().unwrap();
        let val = obj.get("item").unwrap();

        assert_eq!(true, val.is_u64());
        assert_eq!(123u64, val.as_u64().unwrap());
    }

    #[test]
    fn test_get_json_obj() {
        let mut req = Request::new();
        req.payload.extend_from_slice("{ \"item\": 123 }".as_bytes());

        let data: Foo = req.get_json_obj().unwrap();

        assert_eq!(123, data.item);
    }
}
