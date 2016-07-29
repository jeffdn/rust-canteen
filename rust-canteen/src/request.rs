use std::collections::HashMap;

#[allow(dead_code)]
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Method {
    Get,
    Put,
    Post,
    Delete,
    NoImpl,
}

pub trait FromUri {
    fn from_uri(data: &str) -> Self;
}

impl FromUri for String {
    fn from_uri(data: &str) -> String {
        String::from(data)
    }
}

impl FromUri for i32 {
    fn from_uri(data: &str) -> i32 {
        match data.parse::<i32>() {
            Ok(v)  => v,
            Err(e) => panic!("matched integer can't be parsed: {:?}", e),
        }
    }
}

impl FromUri for f32 {
    fn from_uri(data: &str) -> f32 {
        match data.parse::<f32>() {
            Ok(v)  => v,
            Err(e) => panic!("matched float can't be parsed: {:?}", e),
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path:   String,
    pub params: Option<HashMap<String, String>>,
    headers:    HashMap<String, String>,
    payload:    Vec<u8>,
}

impl Request {
    pub fn new() -> Request {
        Request {
            method:  Method::NoImpl,
            path:    String::new(),
            headers: HashMap::new(),
            params:  None,
            payload: Vec::new(),
        }
    }

    fn _get(&self, name: &str) -> Option<&String> {
        let val: Option<&String> = match self.params {
            Some(ref p) => p.get(name),
            None        => None,
        };

        val
    }

    pub fn get<T>(&self, name: &str) -> T where T: FromUri {
        match self._get(&name) {
            Some(item) => FromUri::from_uri(&item),
            None       => panic!("invalid route parameter {:?}", name),
        }
    }

    pub fn from_str(rqstr: &str) -> Request {
        let mut req = Request::new();
        req.parse(rqstr);
        req
    }

    pub fn parse(&mut self, rqstr: &str) {
        let mut buf: Vec<&str> = rqstr.splitn(2, "\r\n").collect();
        let ask: Vec<&str> = buf[0].splitn(3, ' ').collect();

        println!("{:?}", ask);

        self.method = match ask[0] {
            "GET"           => Method::Get,
            "PUT" | "PATCH" => Method::Put,
            "POST"          => Method::Post,
            "DELETE"        => Method::Delete,
            _               => Method::NoImpl,
        };
        self.path = String::from(ask[1]);

        loop {
            buf = buf[1].splitn(2, "\r\n").collect();

            if buf[0] == "" {
                if buf[1] == "" {
                    /* no payload */
                    break;
                }

                let tmp: String;
                buf = buf[1].splitn(2, "\r\n").collect();
                tmp = String::from(buf[1]);
                self.payload.extend(tmp.as_bytes());
                break;
            }

            let hdr: Vec<&str> = buf[0].splitn(2, ": ").collect();

            if hdr.len() == 2 {
                self.headers.insert(String::from(hdr[0]), String::from(hdr[1]));
            }
        }
    }

    pub fn has_params(&self) -> bool {
        match self.params {
            Some(_) => true,
            None    => false,
        }
    }
}
