use std::collections::HashMap;

pub trait ToOutput {
    fn to_output(&self) -> &[u8];
}

impl ToOutput for str {
    fn to_output(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ToOutput for String {
    fn to_output(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ToOutput for Vec<u8> {
    fn to_output(&self) -> &[u8] {
        self.as_slice()
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
        let mut res = Response {
            code:       200,
            cmsg:       String::from("OK"),
            ctype:      String::from("text/plain"),
            headers:    HashMap::new(),
            payload:    Vec::new(),
        };

        res.add_header("Connection", "close");
        res.add_header("Server", "canteen/0.0.1");

        res
    }

    fn get_http_message(code: i32) -> String {
        let msg = match code {
            100 => "Continue",
            101 => "Switching Protocols",
            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            203 => "Non-Authoritative Information",
            204 => "No Content",
            205 => "Reset Content",
            206 => "Partial Content",
            300 => "Multiple Choices",
            301 => "Moved Permanently",
            302 => "Found",
            303 => "See Other",
            304 => "Not Modified",
            305 => "Use Proxy",
            307 => "Temporary Redirect",
            400 => "Bad Request",
            401 => "Unauthorized",
            402 => "Payment Required",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            406 => "Not Acceptable",
            407 => "Proxy Authentication Required",
            408 => "Request Time Out",
            409 => "Conflict",
            410 => "Gone",
            411 => "Length Required",
            412 => "Precondition Failed",
            413 => "Request Entity Too Large",
            414 => "Request-URI Too Large",
            415 => "Unsupported Media Type",
            416 => "Requested Range Not Satisfiable",
            417 => "Expectation Failed",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Time-out",
            505 => "HTTP Version Not Supported",
            _     => "OK",
        };

        String::from(msg)
    }

    pub fn err_403(path: &str) -> Response {
        let mut res = Response::new();

        res.set_code(403);
        res.set_content_type("text/html");
        res.append(format!("<html><head>\
                            <style>body {{ font-family: helvetica, sans-serif; }} p {{ font-size: 14 }}</style>\
                            </head><body><h3>Your request failed</h3><p>forbidden: {}</p></body></html>", path));

        res
    }

    pub fn err_404(path: &str) -> Response {
        let mut res = Response::new();

        res.set_code(403);
        res.set_content_type("text/html");
        res.append(format!("<html><head>\
                            <style>body {{ font-family: helvetica, sans-serif; }} p {{ font-size: 14 }}</style>\
                            </head><body><h3>Your request failed</h3><p>not found: {}</p></body></html>", path));

        res
    }

    /* set the response code
     * ex: res.set_code(200, "OK");
     */
    pub fn set_code(&mut self, code: i32) {
        self.code = code;
        self.cmsg = Response::get_http_message(code);
    }

    /* set the content type
     * ex: res.set_content_type("text/html");
     */
    pub fn set_content_type(&mut self, ctype: &str) {
        self.ctype = String::from(ctype);
    }

    /* add an HTTP header
     * ex: res.add_header("Connection", "close");
     */
    pub fn add_header(&mut self, key: &str, value: &str) {
        if !self.headers.contains_key(key) {
            self.headers.insert(String::from(key), String::from(value));
        }
    }

    /* add data to the payload -- can take any type that has
     * implemented the canteen::response::ToOutput trait
     */
    pub fn append<T: ToOutput>(&mut self, payload: T) {
        self.payload.extend(payload.to_output().into_iter());
    }

    pub fn gen_output(&self) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::new();
        let mut inter = String::new();

        inter.push_str(&format!("HTTP/1.1 {} {}\r\n", self.code, self.cmsg));

        for (key, value) in &self.headers {
            inter.push_str(&format!("{}: {}\r\n", key, value));
        }

        inter.push_str(&format!("Content-Type: {}\r\n", self.ctype));
        inter.push_str(&format!("Content-Length: {}\r\n", self.payload.len()));
        inter.push_str("\r\n");

        output.extend(inter.as_bytes());
        output.extend(self.payload.iter());

        output
    }
}
