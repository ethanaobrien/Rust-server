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
    //let host = res.get_header("Host");
    //res.write_string(("Host: ".to_string() + &host).as_str());
    res.write_string("It works");
    res.end();
    
    //need accept-ranges, type, length, date, keep alive
    
}
