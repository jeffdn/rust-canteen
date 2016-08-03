extern crate canteen;
extern crate rustc_serialize;
extern crate postgres;
extern crate chrono;

use canteen::Canteen;
use canteen::route::*;
use canteen::request::*;
use canteen::response::*;

use rustc_serialize::json;
use rustc_serialize::{Encoder, Encodable};
use rustc_serialize::{Decoder, Decodable};
use postgres::{Connection, SslMode};

type Date = chrono::NaiveDate;

/* a full person record */
#[derive(Debug)]
struct Person {
    id:         i32,
    first_name: String,
    last_name:  String,
    dob:        Date,
}

impl Encodable for Person {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("Person", 4, |s| {
            try!(s.emit_struct_field("id", 0, |s| { s.emit_i32(self.id) }));
            try!(s.emit_struct_field("first_name", 1, |s| { s.emit_str(&self.first_name) }));
            try!(s.emit_struct_field("last_name", 2, |s| { s.emit_str(&self.last_name) }));
            try!(s.emit_struct_field("dob", 3, |s| { s.emit_str(&self.dob.format("%Y-%m-%d").to_string()) }));

            Ok(())
        })
    }
}

impl Decodable for Person {
    fn decode<D: Decoder>(d: &mut D) -> Result<Person, D::Error> {
        d.read_struct("root", 3, |d| {
            let first_name = try!(d.read_struct_field("first_name", 0, |d| { d.read_str() }));
            let last_name = try!(d.read_struct_field("last_name", 0, |d| { d.read_str() }));
            let pre_dob = try!(d.read_struct_field("dob", 0, |d| { d.read_str() }));

            match Date::parse_from_str(&pre_dob, "%Y-%m-%d") {
                Ok(dob) => {
                    Ok(Person {
                        id:         0,
                        first_name: first_name,
                        last_name:  last_name,
                        dob:        dob,
                    })
                },
                Err(_)  => {
                    Err(d.error("failed to parse date provided"))
                },
            }
        })

    }
}

fn create_person(req: &Request) -> Response {
    let mut res = Response::new();
    res.set_content_type("application/json");

    let pers: Person = json::decode(&String::from_utf8(req.payload.clone()).unwrap()).unwrap();

    let conn = Connection::connect("postgresql://jeff@localhost/jeff", SslMode::None).unwrap();
    let cur = conn.query("insert into person (first_name, last_name, dob)\
                          values ($1, $2, $3) returning id",
                          &[&pers.first_name, &pers.last_name, &pers.dob]);

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

    match conn.query("select id, first_name, last_name, dob from person where id = $1", &[&person_id]) {
        Ok(rows)    => {
            match rows.len() {
                1 => {
                    let row = rows.get(0);
                    let p = Person {
                        id:         row.get("id"),
                        first_name: row.get("first_name"),
                        last_name:  row.get("last_name"),
                        dob:        row.get("dob"),
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

fn get_many_person(req: &Request) -> Response {
    let mut res = Response::new();
    res.set_content_type("application/json");

    let conn = Connection::connect("postgresql://jeff@localhost/jeff", SslMode::None).unwrap();
    let cur = conn.query("select id, first_name, last_name, dob from person order by id", &[]);

    match cur {
        Ok(rows)    => {
            let mut people: Vec<Person> = Vec::new();

            for row in rows.iter() {
                people.push(Person {
                    id:         row.get("id"),
                    first_name: row.get("first_name"),
                    last_name:  row.get("last_name"),
                    dob:        row.get("dob"),
                });
            }

            res.append(json::encode(&people).unwrap());
        },
        Err(e)      => {
            res.set_code(500);
            res.append(format!("{{ message: '{:?}' }}", e));
        }
    }

    res
}

fn get_single_person(req: &Request) -> Response {
    let mut res = Response::new();
    res.set_content_type("application/json");

    let person_id: i32 = req.get("person_id");

    let conn = Connection::connect("postgresql://jeff@localhost/jeff", SslMode::None).unwrap();
    let cur = conn.query("select id, first_name, last_name, dob from person where id = $1", &[&person_id]);

    match cur {
        Ok(rows)    => {
            match rows.len() {
                1 => {
                    let row = rows.get(0);
                    let p = Person {
                        id:         row.get("id"),
                        first_name: row.get("first_name"),
                        last_name:  row.get("last_name"),
                        dob:        row.get("dob"),
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
    cnt.add_route("/person", vec![Method::Get], get_many_person);
    cnt.add_route("/person/<int:person_id>", vec![Method::Get], get_single_person);
    cnt.add_route("/src/<path:path>", vec![Method::Get], Route::static_file);
    cnt.set_default(Route::err_404);

    cnt.run();
}

