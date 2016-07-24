extern crate regex;

use Route;
use Method;
use Request;
use regex::Regex;

#[test]
fn test_route_match_fail() {
    let rt = Route::new("/api/v1/foo/<int:foo_id>");
    assert_eq!(false, rt.is_match("/api/v1/bar"));
    assert_eq!(false, rt.is_match("/api/v1/foo/asdf"));
}

#[test]
fn test_route_match_simple() {
    let rt = Route::new("/api/v1/foo/<foo_stuff>");
    assert_eq!("blahblahblah", rt.parse("/api/v1/foo/blahblahblah").unwrap().get("foo_stuff").unwrap());
}

#[test]
fn test_route_match_single_int() {
    let rt = Route::new("/api/v1/foo/<int:foo_id>");
    assert_eq!("123", rt.parse("/api/v1/foo/123").unwrap().get("foo_id").unwrap());
}

#[test]
fn test_route_match_single_str() {
    let rt = Route::new("/api/v1/foo/<str:foo_stuff>");
    assert_eq!("blahblahblah", rt.parse("/api/v1/foo/blahblahblah").unwrap().get("foo_stuff").unwrap());
}

#[test]
fn test_route_match_many() {
    let rt = Route::new("/api/v1/foo/<int:foo_id>/bar/<str:bar>/baz/<int:baz_id>");
    let rm = rt.parse("/api/v1/foo/123/bar/bar/baz/456").unwrap();
    assert_eq!("123", rm.get("foo_id").unwrap());
    assert_eq!("bar", rm.get("bar").unwrap());
    assert_eq!("456", rm.get("baz_id").unwrap());
}

#[test]
fn test_find_route_native_types() {
    let mut request = Request::new();
    let routes: Vec<Route> = vec![Route::new("/api/v1/foo/<int:foo_id>"),
                                  Route::new("/api/v1/foo/<int:foo_id>/bar/<int:bar_id>")];

    request.method = Method::Get;
    request.path = String::from("/api/v1/foo/42/bar/1234");

    for route in routes {
        match route.is_match(&request.path) {
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
