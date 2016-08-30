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
        let re = Regex::new(r"^<(?:(int|str|float|path):)?([\w_][a-zA-Z0-9_]*)>$").unwrap();
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
                                "int"   => ParamType::Integer,
                                "float" => ParamType::Float,
                                "path"  => ParamType::Path,
                                _       => ParamType::String,

                            }
                        }
                        None        => ParamType::String,
                    };

                    let mstr: String = match ptype {
                        ParamType::String  => String::from(r"(?:[^/])+"),
                        ParamType::Integer => String::from(r"-*[0-9]+"),
                        ParamType::Float   => String::from(r"-*[0-9]*[.]?[0-9]+"),
                        ParamType::Path    => String::from(r".+"),
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

        match self.matcher.is_match(&path) {
            true  => {
                let caps = self.matcher.captures(path).unwrap();
                for (param, _) in &self.params {
                    params.insert(param.clone(), String::from(caps.name(&param).unwrap()));
                }
            },
            false => {},
        }

        params
    }
}
