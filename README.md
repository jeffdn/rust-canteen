# Canteen

[![Build Status](https://gitlab.com/jeffdn/rust-canteen/badges/master/build.svg)](https://gitlab.com/jeffdn/rust-canteen/pipelines) [![Latest Version](https://img.shields.io/crates/v/canteen.svg)](https://crates.io/crates/canteen)

## Description

Canteen is the first project that I'm implementing in Rust. It's a simple clone of
[Flask](http://flask.pocoo.org), my very favorite Python web framework. There is code for
an example implementation in the [canteen-impl](https://gitlab.com/jeffdn/canteen-impl)
repository.

## Usage

It's by no means complete, but I'm working on it, and it's now available on
[crates.io](https://crates.io/)! To install and check it out, add the following to
your Cargo.toml:
```toml
[dependencies]
canteen = "0.2"
```

## Example

The principle behind Canteen is simple -- handler functions are defined as simple
Rust functions that take a `Request` and return a `Response`. Handlers are then attached
to one or more routes and HTTP methods/verbs. Routes are specified using a simple
syntax that lets you define variables within them; variables that can then be
extracted to perform various operations. Currently, the following variable types can
be used:

- `<path:name>` will greedily take all path data contained
  - ex: `cnt.add_route("/static/<path:name>", &[Method::Get], utils::static_file)` will
  serve anything in the `/static/` directory as a file
- `<int:name>` will return an integer from a path segment
  - ex: `cnt.add_route("/api/foo/<int:foo_id>", &[Method::Get], my_handler)` will match
  `"/api/foo/123"` but not `"/api/foo/123.34"` or `"/api/foo/bar"`
- `<float:name>` does the same thing as the `int` parameter definition, but matches numbers
with decimal points
- `<str:name>` will match anything inside a path segment, except a forward slash

```rust
extern crate canteen;

use canteen::*;
use canteen::utils;

fn hello_handler(req: &Request) -> Response {
    let mut res = Response::new();

    res.set_code(200);
    res.set_content_type("text/plain");
    res.append("Hello, world!");

    res
}

fn double_handler(req: &Request) -> Response {
    let mut res = Response::new();
    let to_dbl: i32 = req.get("to_dbl");

    /* simpler response generation syntax */
    utils::make_response(format!("{}", to_dbl * 2), "text/plain", 200)
}

fn main() {
    let cnt = Canteen::new(("127.0.0.1", 8080));

    // set the default route handler to show a 404 message
    cnt.set_default(utils::err_404);

    // respond to requests to / with "Hello, world!"
    cnt.add_route("/", &[Method::Get], hello_handler);

    // pull a variable from the path and do something with it
    cnt.add_route("/double/<int:to_dbl>", &[Method::Get], double_handler);

    // serve raw files from the /static/ directory
    cnt.add_route("/static/<path:path>", &[Method::Get], utils::static_file);

    cnt.run();
}
```
