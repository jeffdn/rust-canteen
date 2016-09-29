extern crate canteen;
extern crate hyper;

use canteen::{Canteen, Request, Response, Method};
use canteen::utils;

use hyper::client::Client;

use std::thread;
use std::io::prelude::*;

fn hello_handler(_: &Request) -> Response {
    utils::make_response("Hello, world!", "text/plain", 200)
}

fn double_handler(req: &Request) -> Response {
    let to_dbl: i32 = req.get("to_dbl");
    utils::make_response(format!("{}", to_dbl * 2), "text/plain", 200)
}

fn complex_handler(req: &Request) -> Response {
    let site_id: i32 = req.get("site_id");
    let user_id: i32 = req.get("user_id");
    let prod_id: String = req.get("prod_id");
    let result = format!("{{ site_id: {}, user_id: {}, prod_id: \"{}\" }}", site_id, user_id, prod_id);

    utils::make_response(result, "text/plain", 200)
}

#[test]
fn test_no_route() {
    let th = thread::spawn(|| {
        let mut cnt = Canteen::new(("127.0.0.1", 8881));
        cnt.add_route("/", &[Method::Get], hello_handler);
        cnt.run();
    });

    let client = Client::new();

    let mut res = client.get("http://localhost:8881/").send().unwrap();
    let mut val = String::new();
    let _ = res.read_to_string(&mut val).unwrap();
    assert_eq!("Hello, world!", val);

    let _ = th.join();
}

#[test]
fn test_int_route() {
    let th = thread::spawn(|| {
        let mut cnt = Canteen::new(("127.0.0.1", 8882));
        cnt.add_route("/double/<int:to_dbl>", &[Method::Get], double_handler);
        cnt.run();
    });

    let client = Client::new();

    let mut res = client.get("http://localhost:8882/double/8").send().unwrap();
    let mut val = String::new();
    let _ = res.read_to_string(&mut val).unwrap();
    assert_eq!("16", val);

    let _ = th.join();
}

#[test]
fn test_complex_route() {
    let th = thread::spawn(|| {
        let mut cnt = Canteen::new(("127.0.0.1", 8883));
        cnt.add_route("/api/v1/site/<int:site_id>/user/<int:user_id>/product/<str:prod_id>",
                      &[Method::Get], complex_handler);
        cnt.run();
    });

    let client = Client::new();

    let mut res = client.get("http://localhost:8883/api/v1/site/7/user/382/product/bf73a9cc0").send().unwrap();
    let mut val = String::new();
    let _ = res.read_to_string(&mut val).unwrap();
    assert_eq!("{ site_id: 7, user_id: 382, prod_id: \"bf73a9cc0\" }", val);

    let _ = th.join();
}
