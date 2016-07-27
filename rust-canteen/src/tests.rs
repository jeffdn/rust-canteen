use Route;
use Method;
use Request;
use FromUri;

#[test]
fn test_fromuri_trait_i32() {
    let mut orig: String = String::from("1234");
    let mut conv: i32 = FromUri::from_uri(&orig);

    assert_eq!(1234, conv);

    orig = String::from("-4321");
    conv = FromUri::from_uri(&orig);

    assert_eq!(-4321, conv);
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
    let mut conv: f32 = FromUri::from_uri(&orig);

    assert_eq!(123.45, conv);

    orig = String::from("-54.321");
    conv = FromUri::from_uri(&orig);

    assert_eq!(-54.321, conv);
}

#[test]
fn test_route_match_fail() {
    let rt = Route::new("/api/v1/foo/<int:foo_id>", Route::_no_op);

    assert_eq!(false, rt.is_match("/api/v1/bar"));
    assert_eq!(false, rt.is_match("/api/v1/foo/asdf"));
}

#[test]
fn test_route_match_simple() {
    let route = Route::new("/api/v1/foo/<foo_stuff>", Route::_no_op);
    let parsed = route.parse("/api/v1/foo/blahblahblah").unwrap();

    assert_eq!("blahblahblah", parsed.get("foo_stuff").unwrap());
}

#[test]
fn test_route_match_single_int() {
    let route = Route::new("/api/v1/foo/<int:foo_id>", Route::_no_op);
    let parsed = route.parse("/api/v1/foo/123").unwrap();

    assert_eq!("123", parsed.get("foo_id").unwrap());
}

#[test]
fn test_route_match_single_str() {
    let rt = Route::new("/api/v1/foo/<str:foo_stuff>", Route::_no_op);
    assert_eq!("blahblahblah", rt.parse("/api/v1/foo/blahblahblah").unwrap().get("foo_stuff").unwrap());
}

#[test]
fn test_route_match_many() {
    let rt = Route::new("/api/v1/foo/<int:foo_id>/bar/<str:bar>/baz/<int:baz_id>", Route::_no_op);
    let rm = rt.parse("/api/v1/foo/123/bar/bar/baz/456").unwrap();

    assert_eq!("123", rm.get("foo_id").unwrap());
    assert_eq!("bar", rm.get("bar").unwrap());
    assert_eq!("456", rm.get("baz_id").unwrap());
}

#[test]
fn test_find_route_match() {
    let mut request = Request::new();
    let routes: Vec<Route> = vec![Route::new("/api/v1/foo/<int:foo_id>", Route::_no_op),
                                  Route::new("/api/v1/foo/<int:foo_id>/bar/<int:bar_id>", Route::_no_op)];

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
