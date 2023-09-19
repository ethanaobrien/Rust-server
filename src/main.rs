mod server;
use crate::server::create_server;
use crate::server::Request;

fn main() {
    create_server("127.0.0.1", 8888, on_request);
}

fn on_request(mut res:Request) {
    //let base_path = "C:/Users/ethan/git/EmulatorJS";
    let base_path = "C:/Users/ethan/git/t";
    res.set_header("Connection", "keep-alive");
    res.set_header("Accept-ranges", "bytes");
    
    //todo, should be set by backend
    if res.path.ends_with(".flac") {
        res.set_header("Content-Type", "audio/flac");
    } else {
        res.set_header("Content-Type", "text/html");
    }
    
    //let host = res.get_header("Host");
    //res.write_string(("Host: ".to_string() + &host).as_str());
    if res.method == "PUT" {
        let read = res.read_string(0);
        println!("Got message: {}", read);
    }
    //res.write_string("yes");
    
    res.set_status(200, "OK");
    //println!("{}", success);
    if !success {
        res.write_string("Error");
        res.end();
    }
    //res.end();
    
}
