use crate::server::Request;
use crate::server::create_server;
use crate::server::Settings;


#[allow(dead_code)]
#[allow(unused_assignments)]
pub fn simple_web_server<'a>(opts: Settings<'static>) -> Option<impl FnOnce() + 'a> {
    let host = if opts.local_network {
        if opts.ipv6 { "::" } else { "0.0.0.0" }
    } else {
        if opts.ipv6 { "::1" } else { "127.0.0.1" }
    };
    let result = create_server(host, opts.port, on_request, opts.clone());
    result
}

fn on_request(mut res:Request, opts:Settings) {
    res.write_string(opts.path);
    res.end();
}
