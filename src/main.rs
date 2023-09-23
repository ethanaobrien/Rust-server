mod server;
mod simple_web_server;
use crate::simple_web_server::SimpleWebServer;
use std::thread;
use std::time::Duration;
use crate::server::Settings;

fn main() {
    let settings = Settings {
        port: 8888,
        path: "C:/Users",
        local_network: false,
        spa: false,
        rewrite_to: "",
        directory_listing: true,
        exclude_dot_html: false,
        ipv6: false,
        hidden_dot_files: false,
        cors: false,
        upload: false,
        replace: false,
        delete: false,
        hidden_dot_files_directory_listing: false,
        custom404: "",
        custom403: "",
        custom401: "",
        http_auth: false,
        http_auth_username: "admin",
        http_auth_password: "admin",
    };
    let mut server = SimpleWebServer::new(settings);
    println!("Server started: {}", server.start());
    //let mut i = 0;
    loop {
    //    i += 1;
    //    if i > 20 {
    //        server.terminate();
    //    }
        thread::sleep(Duration::from_millis(100));
    }
}


/*
fn on_request(mut res:Request) {
    //let base_path = "C:/Users/ethan/git/EmulatorJS";
    let base_path = "C:";
    res.set_header("Connection", "keep-alive");
    res.set_header("Accept-ranges", "bytes");
    
    //let host = res.get_header("Host");
    //res.write_string(("Host: ".to_string() + &host).as_str());
    if res.method == "PUT" {
        let read = res.read_string(0);
        println!("Got message: {}", read);
    }
    //res.write_string("yes");
    
    res.set_status(200);
    let success = res.send_file(&(base_path.to_owned() + &url_decode(&res.path.split("?").collect::<Vec<_>>()[0])));
    //println!("{}", success);
    if success == 500 {
        res.set_status(500);
        res.write_string("Error");
    }
    if success == 404 {
        let s2 = res.directory_listing(&(base_path.to_owned() + &url_decode(&res.path.split("?").collect::<Vec<_>>()[0])));
        if s2 == 500 {
            res.set_status(500);
            res.write_string("Error");
        }
        if s2 == 404 {
            res.set_status(404);
            res.write_string("Not found");
        }
        //res.end();
    }
    res.end();
    
}
*/
