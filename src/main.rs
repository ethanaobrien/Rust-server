mod server;
mod simple_web_server;
use crate::simple_web_server::SimpleWebServer;
use std::thread;
use std::time::Duration;
use crate::server::Settings;
use crate::server::generate_dummy_cert_and_key;

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn main() {
    let mut cert = String::new();
    let mut key = String::new();
    
    match generate_dummy_cert_and_key() {
        Ok((certt, keyy)) => {
            cert = certt;
            key = keyy;
        }
        Err(err) => {
            eprintln!("Error generating certificate and key: {:?}", err);
        }
    }
    
    let settings = Settings {
        port: 8888,
        path: "C:/Users",
        local_network: false,
        spa: false,
        rewrite_to: "",
        index: false,
        directory_listing: true,
        exclude_dot_html: false,
        ipv6: false,
        hidden_dot_files: true,
        cors: false,
        upload: false,
        replace: false,
        delete: false,
        hidden_dot_files_directory_listing: true,
        custom500: "",
        custom404: "",
        custom403: "",
        custom401: "",
        http_auth: false,
        http_auth_username: "く",
        http_auth_password: "password",
        https: true,
        https_cert: string_to_static_str(cert),
        https_key: string_to_static_str(key)
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

