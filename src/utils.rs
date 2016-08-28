/* Copyright (c) 2016
 * Jeff Nettleton
 *
 * Licensed under the MIT license (http://opensource.org/licenses/MIT). This
 * file may not be copied, modified, or distributed except according to those
 * terms
 */

use response::{ToOutput, Response};

/* create a response from the basic components */
pub fn make_response<T: ToOutput>(body: T, c_type: &str, code: u16) -> Response {
    let mut res = Response::new();

    res.set_code(code);
    res.set_content_type(c_type);
    res.append(body);

    res
}
