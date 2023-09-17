mod server;
use crate::server::create_server;
use crate::server::Request;

fn main() {
    create_server("127.0.0.1", 8888, on_request);
}

fn on_request(mut res:Request) {
    res.set_header("Connection", "close");
    res.set_header("Content-Type", "text/plain");
    res.set_status(200, "OK");
    res.send_headers();
    res.write_string("Helloooooooooooooooo");
    res.end();
    
    //need accept-ranges, type, length, date, keep alive
    
}
