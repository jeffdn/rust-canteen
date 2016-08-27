use route::*;
use request::*;
use response::*;

#[test]
fn test_response_http_message() {
    assert_eq!("OK", Response::get_http_message(200));
}

#[test]
fn test_tooutput_trait_str() {
    let ar: [u8; 3] = [97, 98, 99];

    assert_eq!(ar, ToOutput::to_output("abc"));
}

#[test]
fn test_tooutput_trait_string() {
    let ar: [u8; 3] = [97, 98, 99];
    let st = String::from("abc");

    assert_eq!(ar, ToOutput::to_output(&st));
}

#[test]
fn test_tooutput_trait_vec() {
    let ar: [u8; 5] = [1, 2, 3, 4, 5];
    let vc: Vec<u8> = vec![1, 2, 3, 4, 5];

    assert_eq!(ar, ToOutput::to_output(&vc));
}

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

#[test]
fn test_route_match_fail() {
    let rt = Route::new("/api/v1/foo/<int:foo_id>", Method::Get, Route::err_404);
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
    let route = Route::new("/api/v1/foo/<foo_stuff>", Method::Get, Route::err_404);
    let parsed = route.parse("/api/v1/foo/blahblahblah");

    assert_eq!("blahblahblah", parsed.get("foo_stuff").unwrap());
}

#[test]
fn test_route_match_single_int() {
    let route = Route::new("/api/v1/foo/<int:foo_id>", Method::Get, Route::err_404);
    let parsed = route.parse("/api/v1/foo/123");

    assert_eq!("123", parsed.get("foo_id").unwrap());
}

#[test]
fn test_route_match_single_str() {
    let rt = Route::new("/api/v1/foo/<str:foo_stuff>", Method::Get, Route::err_404);
    assert_eq!("blahblahblah", rt.parse("/api/v1/foo/blahblahblah").get("foo_stuff").unwrap());
}

#[test]
fn test_route_match_many() {
    let rt = Route::new("/api/v1/foo/<int:foo_id>/bar/<str:bar>/baz/<int:baz_id>", Method::Get, Route::err_404);
    let rm = rt.parse("/api/v1/foo/123/bar/bar/baz/456");

    assert_eq!("123", rm.get("foo_id").unwrap());
    assert_eq!("bar", rm.get("bar").unwrap());
    assert_eq!("456", rm.get("baz_id").unwrap());
}

#[test]
fn test_find_route_native_types() {
    let mut request = Request::new();
    let routes: Vec<Route> = vec![Route::new("/api/v1/foo/<int:foo_id>", Method::Get, Route::err_404),
                                  Route::new("/api/v1/foo/<int:foo_id>/bar/<int:bar_id>", Method::Get, Route::err_404)];

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
