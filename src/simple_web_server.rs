use crate::server::Request;
use crate::server::create_server;
use crate::server::Settings;

#[allow(dead_code)]
#[allow(unused_assignments)]
pub struct SimpleWebServer<'a> {
    opts: Settings<'a>,
    error: bool
}

#[allow(dead_code)]
#[allow(unused_assignments)]
impl SimpleWebServer<'_> {
    pub fn new(opts: Settings<'static>) -> SimpleWebServer {
        let mut error = false;
        let host = if opts.local_network {
            if opts.ipv6 { "::" } else { "0.0.0.0" }
        } else {
            if opts.ipv6 { "::1" } else { "127.0.0.1" }
        };
        let result = create_server(host, opts.port, on_request, opts.clone());
        if let Some(kill) = result {
            //kill(); // Now just how do I get this to the terminate function...
        } else {
            error = true;
        }
        SimpleWebServer {
            opts: opts,
            error: error
        }
    }
    pub fn terminate(&self) {
        
    }
}

fn on_request(mut res:Request, opts:Settings) {
    res.write_string(opts.path);
    res.end();
}
