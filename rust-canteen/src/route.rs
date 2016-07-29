extern crate regex;

use std::collections::HashMap;
use std::collections::HashSet;
use regex::Regex;

use request::*;
use response::*;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum ParamType {
    Integer,
    String,
    Float,
}

#[derive(Debug)]
pub struct Route {
    pathdef:     String,
    matcher:     Regex,
    methods:     HashSet<Method>,
    params:      HashMap<String, ParamType>,
    pub handler: fn(Request) -> Response,
}

impl Route {
    pub fn new(path: &str, mlist: Vec<Method>, handler: fn(Request) -> Response) -> Route {
        let re = Regex::new(r"^<(?:(int|str):)?([\w_][a-zA-Z0-9_]*)>$").unwrap();
        let parts: Vec<&str> = path.split('/').filter(|&s| s != "").collect();
        let mut matcher: String = String::from(r"^");
        let mut params: HashMap<String, ParamType> = HashMap::new();
        let mut methods: HashSet<Method> = HashSet::new();

        for m in mlist {
            methods.insert(m);
        }

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
            methods: methods,
            handler: handler,
        }
    }

    pub fn is_match(&self, req: &Request) -> bool {
        self.matcher.is_match(&req.path) && self.methods.contains(&req.method)
    }

    pub fn parse(&self, path: &str) -> Option<HashMap<String, String>> {
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

    pub fn _no_op(req: Request) -> Response {
        let mut res = Response::new();

        res.append(req.path);

        res
    }

    pub fn err_403(req: Request) -> Response {
        Response::err_403(&req.path)
    }

    pub fn err_404(req: Request) -> Response {
        Response::err_404(&req.path)
    }
}
