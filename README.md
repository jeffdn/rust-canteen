# rust-stuff

This project contains my attempts to learn Rust, an exciting new programming language
being sponsored by the Mozilla Foundation. Given that my first language was C, I had
long desired learning another systems-level language that was not in the C familiy.

## Canteen

Canteen is the first project that I'm implementing in Rust. It's a simple clone of
[Flask](http://flask.pocoo.org), my very favorite Python web framework. The code for
the library is located above in the `rust-canteen` directory, and there is code for
an example implementation in the `canteen-impl` directory. Here's a simple example:
```rust
extern crate canteen;

use canteen::Canteen;
use canteen::request::*;
use canteen::response::*;

fn handler(req: &Request) -> Response {
    let mut res = Response::new();

    res.set_content_type("text/plain");
    res.append("Hello, world!");

    res
}

fn main() {
    let cnt = Canteen::new(("127.0.0.1", 8080));

    cnt.add_route("/", vec![Method::Get], handler);
    cnt.run();
}
```
It's by no means complete, but I'm working on it! To install and check it out, add
the following to your Cargo.toml:
```toml
[dependencies]
canteen = { version = "0.0.1", path = "/path/to/rust-stuff/rust-canteen" }
```
