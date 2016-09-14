// Copyright (c) 2016
// Jeff Nettleton
//
// Licensed under the MIT license (http://opensource.org/licenses/MIT). This
// file may not be copied, modified, or distributed except according to those
// terms

use std::collections::HashMap;

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

    /// Get a variable from the URI.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Given the route "/hello/<str:name>"
    /// fn handler(req: &Request) -> Response {
    ///     make_response(format!("<b>Hello, {}!</b>", req.get("name")), "text/html", 200)
    /// }
    /// ```
    pub fn get<T: FromUri>(&self, name: &str) -> T {
        match self.params.get(name) {
            Some(item) => FromUri::from_uri(&item),
            None       => panic!("invalid route parameter {:?}", name),
        }
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

    #[test]
    fn test_fromuri_trait_i32() {
        let mut orig: String = String::from("1234");
        assert_eq!(1234, FromUri::from_uri(&orig));

        orig = String::from("-4321");
        assert_eq!(-4321, FromUri::from_uri(&orig));
    }

    #[test]
    fn test_fromuri_trait_string() {
        let orig: String = String::from("foobar");
        let conv: String = FromUri::from_uri(&orig);

        assert_eq!("foobar", conv);
    }

    #[test]
    fn test_fromuri_trait_float() {
        let mut orig: String = String::from("123.45");
        assert_eq!(123.45f32, FromUri::from_uri(&orig));

        orig = String::from("-54.321");
        assert_eq!(-54.321f32, FromUri::from_uri(&orig));
    }
}
