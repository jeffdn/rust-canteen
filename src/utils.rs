// Copyright (c) 2016
// Jeff Nettleton
//
// Licensed under the MIT license (http://opensource.org/licenses/MIT). This
// file may not be copied, modified, or distributed except according to those
// terms

use std::env;
use std::fs::File;
use std::error::Error;
use std::path::PathBuf;
use std::io::prelude::*;
use response::{ToOutput, Response};
use context::Context;

/// Convenience method for creating a response from the basic components
/// required (a request body, content type, and response code).
///
/// # Examples
///
/// ```rust
/// use canteen::{Context, Response};
/// use canteen::utils;
///
/// fn handler(_: &Context) -> Response {
///     utils::make_response("Hello, world!", "text/plain", 200)
/// }
/// ```
pub fn make_response<T: ToOutput>(body: T, c_type: &str, code: u16) -> Response {
    let mut res = Response::new();

    res.set_code(code);
    res.set_content_type(c_type);
    res.append(body);

    res
}

/// Replace the URI escape codes with their ASCII equivalents.
pub fn replace_escape(path: &str) -> String {
    let mut fixed = String::from(path);
    let replaces: [(&str, &str); 22] = [
            ("%20", " "),
            ("%3C", "<"),
            ("%3E", ">"),
            ("%23", "#"),
            ("%25", "%"),
            ("%7B", "["),
            ("%7D", ")"),
            ("%7C", "|"),
            ("%5C", "\\"),
            ("%5E", "^"),
            ("%7E", "~"),
            ("%5B", "["),
            ("%5D", ")"),
            ("%60", "`"),
            ("%3B", ";"),
            ("%2F", "/"),
            ("%3F", "?"),
            ("%3A", ":"),
            ("%40", "@"),
            ("%3D", "="),
            ("%26", "&"),
            ("%24", "$"),
    ];

    for &(from, to) in replaces.iter() {
        fixed = fixed.replace(from, to);
    }

    fixed
}

fn err_body(message: &str, path: &str) -> String {
    format!("<html><head>\
             <style>body {{ font-family: helvetica, sans-serif; }} p {{ font-size: 14 }}</style>\
             </head><body><h3>Your request failed</h3><p>{}: {}</p></body></html>", message, path)
}

/// Default handler function for HTTP 403 errors.
pub fn err_403(ctx: &Context) -> Response {
    make_response(err_body("forbidden", &ctx.request.path), "text/html", 403)
}

/// Default handler function for HTTP 404 errors.
pub fn err_404(ctx: &Context) -> Response {
    make_response(err_body("not found", &ctx.request.path), "text/html", 404)
}

/// Default handler function for HTTP 500 errors.
pub fn err_500(ctx: &Context) -> Response {
    make_response(err_body("internal server error", &ctx.request.path), "text/html", 500)
}

/// Handler that sends static files relative to the current working directory.
pub fn static_file(ctx: &Context) -> Response {
    let mut res = Response::new();

    let cwd = env::current_dir().unwrap();
    let clean = replace_escape(&ctx.request.path);
    let mut fpath = PathBuf::from(&cwd);
    let mut fbuf: Vec<u8> = Vec::new();

    for chunk in clean.split('/') {
        if chunk == "" || chunk == "." || chunk == ".." {
            /* bzzzzt */
            continue;
        }

        fpath.push(&chunk);
    }

    let file = File::open(&fpath);

    match file {
        Ok(mut f)   => {
            match f.read_to_end(&mut fbuf) {
                Ok(_)   => {
                    res.set_code(200);
                    res.set_content_type("text/plain");
                    res.append(fbuf);
                },
                Err(e)  => {
                    return make_response(err_body("internal server error", e.description()), "text/html", 500);
                },
            }
        },
        Err(_)      => {
            return err_404(&ctx);
        }
    }

    res
}
