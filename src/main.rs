mod server;
use crate::server::create_server;
use crate::server::Request;
use crate::server::url_decode;

fn main() {
    create_server("127.0.0.1", 8888, on_request);
}

fn on_request(mut res:Request) {
    //let base_path = "C:/Users/ethan/git/EmulatorJS";
    let base_path = "C:/Users/ethan/Downloads";
    res.set_header("Connection", "keep-alive");
    res.set_header("Accept-ranges", "bytes");
    
    //let host = res.get_header("Host");
    //res.write_string(("Host: ".to_string() + &host).as_str());
    if res.method == "PUT" {
        let read = res.read_string(0);
        println!("Got message: {}", read);
    }
    //res.write_string("yes");
    
    res.set_status(200, "OK");
    let success = res.send_file(&(base_path.to_owned() + &url_decode(&res.path.split("?").collect::<Vec<_>>()[0])));
    //println!("{}", success);
    if !success {
        let s2 = res.directory_listing(&(base_path.to_owned() + &url_decode(&res.path.split("?").collect::<Vec<_>>()[0])));
        if !s2 {
            //is this 404?
            res.set_status(500, "Internal Server Error");
            res.write_string("Error");
        }
        //res.end();
    }
    res.end();
    
}
