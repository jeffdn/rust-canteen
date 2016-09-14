extern crate canteen;
extern crate hyper;

use canteen::{Canteen, Request, Response, Method};
use canteen::utils;

use hyper::client::Client;

use std::thread;
use std::io::prelude::*;

fn hello_handler(_: &Request) -> Response {
    let mut res = Response::new();

    res.set_code(200);
    res.set_content_type("text/plain");
    res.append("Hello, world!");

    res
}

fn double_handler(req: &Request) -> Response {
    let mut res = Response::new();
    let to_dbl: i32 = req.get("to_dbl");

    res.set_code(200);
    res.set_content_type("text/plain");
    res.append(format!("{}", to_dbl * 2));

    res
}

#[test]
#[should_panic]
fn main() {
    let th = thread::spawn(|| {
        let mut cnt = Canteen::new(("127.0.0.1", 8888));

        cnt.add_route("/", &[Method::Get], hello_handler)
           .add_route("/double/<int:to_dbl>", &[Method::Get], double_handler)
           .set_default(utils::err_404);

        cnt._test();
    });

    let client = Client::new();

    let mut res_double = client.get("http://localhost:8888/double/8").send().unwrap();
    let mut val_double = String::new();
    let _ = res_double.read_to_string(&mut val_double).unwrap();
    assert_eq!("16", val_double);

    let mut res_hello = client.get("http://localhost:8888/").send().unwrap();
    let mut val_hello = String::new();
    let _ = res_hello.read_to_string(&mut val_hello).unwrap();
    assert_eq!("Hello, world!", val_hello);

    let _ = client.get("http://localhost:8888/__kill").send().unwrap();
    let _ = th.join();
}
