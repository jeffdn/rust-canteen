/* Copyright (c) 2016
 * Jeff Nettleton
 *
 * Licensed under the MIT license (http://opensource.org/licenses/MIT). This
 * file may not be copied, modified, or distributed except according to those
 * terms
 */

extern crate regex;

use std::collections::HashMap;
use regex::Regex;

use request::*;
use response::*;

// The various types of parameters that can be contained in a URI.
#[derive(PartialEq, Eq, Hash, Debug)]
enum ParamType {
    Integer,
    Unsigned,
    String,
    Float,
    Path,
}

/// This struct represents a route definition. It is only necessary for
/// use internally.
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct RouteDef {
    pub pathdef: String,
    pub method:  Method,
}

/// This struct defines a route or endpoint.
pub struct Route {
    matcher:     Regex,
    method:      Method,
    params:      HashMap<String, ParamType>,
    pub handler: fn(&Request) -> Response,
}

impl Route {
    /// Create a new Route. This function is called by the Canteen struct.
    pub fn new(path: &str, method: Method, handler: fn(&Request) -> Response) -> Route {
        let re = Regex::new(r"^<(?:(int|uint|str|float|path):)?([\w_][a-zA-Z0-9_]*)>$").unwrap();
        let parts: Vec<&str> = path.split('/').filter(|&s| s != "").collect();
        let mut matcher: String = String::from(r"^");
        let mut params: HashMap<String, ParamType> = HashMap::new();

        for part in parts {
            let chunk: String = match re.is_match(part) {
                true  => {
                    let caps = re.captures(part).unwrap();
                    let param = caps.get(2).unwrap().as_str();
                    let ptype: ParamType = match caps.get(1) {
                        Some(x)     => {
                            match x.as_str() {
                                "int"   => ParamType::Integer,
                                "uint"  => ParamType::Unsigned,
                                "float" => ParamType::Float,
                                "path"  => ParamType::Path,
                                "str"   => ParamType::String,
                                _       => ParamType::String,

                            }
                        }
                        None        => ParamType::String,
                    };

                    let mstr: String = match ptype {
                        ParamType::String   => String::from(r"(?:[^/])+"),
                        ParamType::Integer  => String::from(r"-*[0-9]+"),
                        ParamType::Unsigned => String::from(r"[0-9]+"),
                        ParamType::Float    => String::from(r"-*[0-9]*[.]?[0-9]+"),
                        ParamType::Path     => String::from(r".+"),
                    };

                    params.insert(String::from(param), ptype);

                    format!("/(?P<{}>{})", &param, &mstr)
                },
                false => String::from("/") + part,
            };

            matcher.push_str(&chunk);
        }

        /* end the regex with an optional final slash and a string terminator */
        matcher.push_str("/?$");

        Route {
            matcher: Regex::new(&matcher).unwrap(),
            params:  params,
            method:  method,
            handler: handler,
        }
    }

    /// Check if this Route matches a given URI.
    pub fn is_match(&self, req: &Request) -> bool {
        self.matcher.is_match(&req.path) && self.method == req.method
    }

    /// Parse and extract the variables from a URI based on this Route's definition.
    pub fn parse(&self, path: &str) -> HashMap<String, String> {
        let mut params: HashMap<String, String> = HashMap::new();

        if self.matcher.is_match(&path) {
            let caps = self.matcher.captures(path).unwrap();
            for (param, _) in &self.params {
                params.insert(param.clone(), String::from(caps.name(&param).unwrap().as_str()));
            }
        }

        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils;

    #[test]
    fn test_route_match_fail() {
        let rt = Route::new("/api/v1/foo/<int:foo_id>", Method::Get, utils::err_404);
        let mut req = Request::new();

        req.path = String::from("/api/v1/bar");
        req.method = Method::Get;

        assert_eq!(false, rt.is_match(&req));

        req.path = String::from("/api/v1/foo");
        req.method = Method::Post;

        assert_eq!(false, rt.is_match(&req));

        req.path = String::from("/api/v1/foo/asdf");
        req.method = Method::Get;

        assert_eq!(false, rt.is_match(&req));

        req.path = String::from("/api/v1/foo/123");
        req.method = Method::Get;

        assert_eq!(true, rt.is_match(&req));
    }

    #[test]
    fn test_route_match_simple() {
        let route = Route::new("/api/v1/foo/<foo_stuff>", Method::Get, utils::err_404);
        let parsed = route.parse("/api/v1/foo/blahblahblah");

        assert_eq!("blahblahblah", parsed.get("foo_stuff").unwrap());
    }

    #[test]
    fn test_route_match_single_int() {
        let route = Route::new("/api/v1/foo/<int:foo_id>", Method::Get, utils::err_404);
        let parsed = route.parse("/api/v1/foo/123");

        assert_eq!("123", parsed.get("foo_id").unwrap());
    }

    #[test]
    fn test_route_match_single_uint() {
        let route = Route::new("/api/v1/foo/<uint:foo_id>", Method::Get, utils::err_404);
        let parsed = route.parse("/api/v1/foo/123");
        let mut badreq = Request::new();

        badreq.method = Method::Get;
        badreq.path = String::from("/api/v1/foo/-123");

        assert_eq!("123", parsed.get("foo_id").unwrap());
        assert_eq!(false, route.is_match(&badreq));
    }

    #[test]
    fn test_route_match_single_str() {
        let rt = Route::new("/api/v1/foo/<str:foo_stuff>", Method::Get, utils::err_404);
        assert_eq!("blahblahblah", rt.parse("/api/v1/foo/blahblahblah").get("foo_stuff").unwrap());
    }

    #[test]
    fn test_route_match_many() {
        let rt = Route::new("/api/v1/foo/<int:foo_id>/bar/<str:bar>/baz/<int:baz_id>", Method::Get, utils::err_404);
        let rm = rt.parse("/api/v1/foo/123/bar/bar/baz/456");

        assert_eq!("123", rm.get("foo_id").unwrap());
        assert_eq!("bar", rm.get("bar").unwrap());
        assert_eq!("456", rm.get("baz_id").unwrap());
    }

    #[test]
    fn test_find_route_native_types() {
        let mut request = Request::new();
        let routes = vec![Route::new("/api/v1/foo/<int:foo_id>", Method::Get, utils::err_404),
                          Route::new("/api/v1/foo/<int:foo_id>/bar/<int:bar_id>", Method::Get, utils::err_404)];

        request.method = Method::Get;
        request.path = String::from("/api/v1/foo/42/bar/1234");

        for route in routes {
            match route.is_match(&request) {
                false => continue,
                true  => {
                    request.params = route.parse(&request.path);
                    break;
                },
            }
        }

        let foo_id: i32 = request.get("foo_id");
        let bar_id: i32 = request.get("bar_id");

        assert_eq!(42, foo_id);
        assert_eq!(1234, bar_id);
    }
}
