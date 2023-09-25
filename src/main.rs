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
        spa: false,//todo
        rewrite_to: "",//todo
        index: false,
        directory_listing: true,
        exclude_dot_html: false,
        ipv6: false,
        hidden_dot_files: false,//todo
        cors: false,
        upload: false,//todo
        replace: false,//todo
        delete: false,//todo
        hidden_dot_files_directory_listing: false,//todo
        custom404: "",//todo
        custom403: "",//todo
        custom401: "",//todo
        http_auth: false,//todo
        http_auth_username: "admin",//todo
        http_auth_password: "admin",//todo
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
