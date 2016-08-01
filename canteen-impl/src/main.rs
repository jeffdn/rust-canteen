extern crate canteen;
extern crate rustc_serialize;
extern crate postgres;
extern crate chrono;

use canteen::Canteen;
use canteen::route::*;
use canteen::request::*;
use canteen::response::*;

use rustc_serialize::json;
use postgres::{Connection, SslMode};

#[derive(RustcEncodable, RustcDecodable, Debug)]
struct Person {
    id:         i32,
    first_name: String,
    last_name:  String,
}

fn create_person(req: &Request) -> Response {
    let mut res = Response::new();
    let pers: Person = json::decode(&String::from_utf8(req.payload.clone()).unwrap()).unwrap();

    println!("got a person! {:?}", pers);

    let conn = Connection::connect("postgresql://jeff@localhost/jeff", SslMode::None).unwrap();
    let cur = conn.query("insert into people (first_name, last_name)\
                          values ($1, $2) returning id",
                          &[&pers.first_name, &pers.last_name]);

    let person_id: i32;

    match cur {
        Ok(rows)    => {
            match rows.len() {
                1 => {
                    person_id = rows.get(0).get("id");
                },
                _ => {
                    res.set_code(500);
                    res.append(String::from("{ message: 'person couldn\'t be created' }"));
                    return res;
                },
            }
        },
        Err(e)      => {
            res.set_code(500);
            res.append(format!("{{ message: '{:?}' }}", e));
            return res;
        }
    }

    match conn.query("select id, first_name, last_name from people where id = $1", &[&person_id]) {
        Ok(rows)    => {
            match rows.len() {
                1 => {
                    let row = rows.get(0);
                    let p = Person {
                        id:         row.get("id"),
                        first_name: row.get("first_name"),
                        last_name:  row.get("last_name"),
                    };

                    res.append(json::encode(&p).unwrap());
                },
                _ => {
                    res.set_code(404);
                    res.append(String::from("{ message: 'not found' }"));
                },
            }
        },
        Err(e)      => {
            res.set_code(500);
            res.append(format!("{{ message: '{:?}' }}", e));
        }
    }

    res
}

fn get_person(req: &Request) -> Response {
    let mut res = Response::new();
    let person_id: i32 = req.get("person_id");

    let conn = Connection::connect("postgresql://jeff@localhost/jeff", SslMode::None).unwrap();
    let cur = conn.query("select id, first_name, last_name, dob from people where id = $1", &[&person_id]);

    res.set_content_type("application/json");

    match cur {
        Ok(rows)    => {
            match rows.len() {
                1 => {
                    let row = rows.get(0);
                    let p = Person {
                        id:         row.get("id"),
                        first_name: row.get("first_name"),
                        last_name:  row.get("last_name"),
                    };

                    res.append(json::encode(&p).unwrap());
                },
                _ => {
                    res.set_code(404);
                    res.append(String::from("{ message: 'not found' }"));
                },
            }
        },
        Err(e)      => {
            res.set_code(500);
            res.append(format!("{{ message: '{:?}' }}", e));
        }
    }

    res
}

fn main() {
    let mut cnt = Canteen::new(("127.0.0.1", 8080));

    cnt.add_route("/person", vec![Method::Post], create_person);
    cnt.add_route("/person/<int:person_id>", vec![Method::Get], get_person);
    cnt.set_default(Route::err_404);

    cnt.run();
}

