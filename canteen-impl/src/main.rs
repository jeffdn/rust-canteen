extern crate canteen;

use canteen::Canteen;
use canteen::route::*;
use canteen::request::*;
use canteen::response::*;

fn my_handler(req: &Request) -> Response {
    let mut res = Response::new();

    res.set_content_type("text/html");
    res.append(String::from("<html><head>\
                             <style>body { font-family: helvetica, sans-serif; } p { font-size: 14px; }</style>\
                             </head><body><h3>It's alive!</h3><p>Welcome to Canteen! :)</p></body></html>"));

    res
}

fn main() {
    let mut cnt = Canteen::new(("127.0.0.1", 8080));
    cnt.add_route("/hello", vec![Method::Get], my_handler);
    cnt.set_default(Route::err_404);
    cnt.run();
}

