use std::collections::HashMap;

pub trait ToOutput {
    fn to_output(&self) -> &[u8];
}

impl ToOutput for String {
    fn to_output(&self) -> &[u8] {
        self.as_bytes()
    }
}

#[derive(Debug)]
pub struct Response {
    code:       i32,
    cmsg:       String,
    ctype:      String,
    headers:    HashMap<String, String>,
    payload:    Vec<u8>,
}

impl Response {
    pub fn new() -> Response {
        Response {
            code:       200,
            cmsg:       String::new(),
            ctype:      String::from("text/plain"),
            headers:    HashMap::new(),
            payload:    Vec::new(),
        }
    }

    pub fn set_code(&mut self, code: i32, cmsg: &str) {
        self.code = code;
        self.cmsg = String::from(cmsg);
    }

    pub fn set_content_type(&mut self, ctype: &str) {
        self.ctype = String::from(ctype);
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.insert(String::from(key), String::from(value));
    }

    pub fn append<T: ToOutput>(&mut self, payload: T) {
        self.payload.extend(payload.to_output().into_iter());
    }

    pub fn gen_output(&self) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::new();
        let mut inter = String::new();

        inter.push_str(&format!("HTTP/1.1 {} {}\r\n", self.code, self.cmsg));
        inter.push_str("Connection: close\r\n");
        inter.push_str("Server: canteen/0.0.1\r\n");
        inter.push_str(&format!("Content-Type: {}\r\n", self.ctype));
        inter.push_str(&format!("Content-Length: {}\r\n", self.payload.len()));
        inter.push_str("\r\n\r\n");

        output.extend(inter.as_bytes());
        output.extend(self.payload.iter());

        output
    }
}
