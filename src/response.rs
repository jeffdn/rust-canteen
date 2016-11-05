// Copyright (c) 2016
// Jeff Nettleton
//
// Licensed under the MIT license (http://opensource.org/licenses/MIT). This
// file may not be copied, modified, or distributed except according to those
// terms

use std::collections::HashMap;
use chrono::UTC;
use rustc_serialize::{json, Encodable};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// A trait that converts data from the handler function to a u8 slice.
pub trait ToOutput {
    fn to_output(&self) -> &[u8];
}

impl ToOutput for str {
    fn to_output(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ToOutput for &'static str {
    fn to_output(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ToOutput for String {
    fn to_output(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ToOutput for Vec<u8> {
    fn to_output(&self) -> &[u8] {
        self.as_slice()
    }
}

/// This struct reprsents the response to an HTTP client.
#[derive(Debug)]
pub struct Response {
    status:     u16,
    cmsg:       String,
    ctype:      String,
    headers:    HashMap<String, String>,
    payload:    Vec<u8>,
}

impl Response {
    /// Create a new, empty Response.
    pub fn new() -> Response {
        let mut res = Response {
            status:     200,
            cmsg:       String::from("OK"),
            ctype:      String::from("text/plain"),
            headers:    HashMap::new(),
            payload:    Vec::with_capacity(2048),
        };

        let now = UTC::now().format("%a, %d %b %Y, %H:%M:%S %Z").to_string();

        res.add_header("Connection", "close");
        res.add_header("Server", &format!("canteen/{}", VERSION));
        res.add_header("Date", &now);

        res
    }

    /// Creates a Response with a JSON body
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use canteen::Response;
    ///
    /// #[derive(RustcEncodable)]
    /// struct Foo {
    ///     item: i32,
    /// }
    ///
    /// let foo = Foo { item: 12345 };
    /// let res = Response::as_json(&foo);
    /// ```
    pub fn as_json<T: Encodable>(data: &T) -> Response {
        let mut res = Response::new();

        res.set_content_type("application/json");
        res.append(json::encode(data).unwrap());

        res
    }

    /// Gets the HTTP message for a given status.
    fn get_http_message(status: u16) -> String {
        let msg = match status {
            100 => "Continue",
            101 => "Switching Protocols",
            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            203 => "Non-Authoritative Information",
            204 => "No Content",
            205 => "Reset Content",
            206 => "Partial Content",
            300 => "Multiple Choices",
            301 => "Moved Permanently",
            302 => "Found",
            303 => "See Other",
            304 => "Not Modified",
            305 => "Use Proxy",
            307 => "Temporary Redirect",
            400 => "Bad Request",
            401 => "Unauthorized",
            402 => "Payment Required",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            406 => "Not Acceptable",
            407 => "Proxy Authentication Required",
            408 => "Request Time Out",
            409 => "Conflict",
            410 => "Gone",
            411 => "Length Required",
            412 => "Precondition Failed",
            413 => "Request Entity Too Large",
            414 => "Request-URI Too Large",
            415 => "Unsupported Media Type",
            416 => "Requested Range Not Satisfiable",
            417 => "Expectation Failed",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Time-out",
            505 => "HTTP Version Not Supported",
            _   => "OK",
        };

        String::from(msg)
    }

    /// Sets the response status for the HTTP response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use canteen::Response;
    ///
    /// let mut res = Response::new();
    /// res.set_status(200);
    /// ```
    pub fn set_status(&mut self, status: u16) {
        self.status = status;
        self.cmsg = Response::get_http_message(status);
    }

    /// Sets the Content-Type header for the HTTP response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use canteen::Response;
    ///
    /// let mut res = Response::new();
    /// res.set_content_type("text/html");
    /// ```
    pub fn set_content_type(&mut self, ctype: &str) {
        self.ctype = String::from(ctype);
    }

    /// Adds a header to the HTTP response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use canteen::Response;
    ///
    /// let mut res = Response::new();
    /// res.add_header("Content-Type", "text/html");
    /// ```
    pub fn add_header(&mut self, key: &str, value: &str) {
        if !self.headers.contains_key(key) {
            self.headers.insert(String::from(key), String::from(value));
        }
    }

    /// Appends data to the body of the HTTP response. The trait ToOutput must
    /// be implemented for the type passed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use canteen::Response;
    ///
    /// let mut res = Response::new();
    /// let data = "{ message: \"Hello, world!\" }";
    /// res.append(data);
    /// ```
    pub fn append<T: ToOutput>(&mut self, payload: T) {
        self.payload.extend(payload.to_output().into_iter());
    }

    /// Returns a byte array containing the full contents of the HTTP response,
    /// for use by the Canteen struct.
    pub fn gen_output(&self) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::with_capacity(self.payload.len() + 500);
        let mut inter = String::new();

        inter.push_str(&format!("HTTP/1.1 {} {}\r\n", self.status, self.cmsg));

        for (key, value) in &self.headers {
            inter.push_str(&format!("{}: {}\r\n", key, value));
        }

        inter.push_str(&format!("Content-Type: {}\r\n", self.ctype));
        inter.push_str(&format!("Content-Length: {}\r\n", self.payload.len()));
        inter.push_str("\r\n");

        output.extend(inter.as_bytes());
        output.extend(self.payload.iter());

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_serialize::json;

    #[derive(RustcEncodable)]
    struct Foo {
        item: i32,
    }

    #[test]
    fn test_response_as_json() {
        let foo = Foo { item: 12345 };
        let res_j = Response::as_json(&foo);
        let mut res_r = Response::new();

        res_r.set_content_type("application/json");
        res_r.append(json::encode(&foo).unwrap());

        assert_eq!(res_r.gen_output(), res_j.gen_output());
    }

    #[test]
    fn test_response_http_message() {
        assert_eq!("OK", Response::get_http_message(200));
    }

    #[test]
    fn test_tooutput_trait_static_str() {
        let ar: [u8; 3] = [97, 98, 99];

        assert_eq!(ar, "abc".to_output());
    }

    #[test]
    fn test_tooutput_trait_str() {
        let ar: [u8; 3] = [97, 98, 99];
        let st = "abc";

        assert_eq!(ar, st.to_output());
    }

    #[test]
    fn test_tooutput_trait_string() {
        let ar: [u8; 3] = [97, 98, 99];
        let st = String::from("abc");

        assert_eq!(ar, st.to_output());
    }

    #[test]
    fn test_tooutput_trait_vec() {
        let ar: [u8; 5] = [1, 2, 3, 4, 5];
        let vc: Vec<u8> = vec![1, 2, 3, 4, 5];

        assert_eq!(ar, vc.to_output());
    }
}
