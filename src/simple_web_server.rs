use crate::server::Server;
use crate::server::Settings;
use crate::server::file_system::GetByPath;
use crate::server::Request;

pub struct SimpleWebServer {
    server: Server
}

#[allow(dead_code)]
impl SimpleWebServer {
    pub fn new(opts: Settings<'static>) -> SimpleWebServer {
        SimpleWebServer {
            server: Server::new(opts, SimpleWebServer::on_request)
        }
    }
    pub fn start(&mut self) -> bool {
        return self.server.start();
    }
    pub fn terminate(&mut self) {
        return self.server.terminate();
    }
    fn on_request(mut res:Request, opts: Settings) {
        //todo, this thing
        res.set_header("Connection", "keep-alive");
        res.set_header("Accept-ranges", "bytes");
        
        if opts.cors {
            res.set_header("access-control-allow-origin", "*");
            res.set_header("access-control-allow-methods", "GET, POST, PUT, DELETE");
            res.set_header("access-control-max-age", "120");
        }
        
        if res.method == "GET" || res.method == "HEAD" {
            SimpleWebServer::get(res, opts);
        } else {
            res.set_header("Content-length", "0");
            res.set_status(501);
            res.end();
        }
        
    }
    fn get(mut res:Request, opts: Settings) {
        let path = res.origpath.clone();
        let mut file_path = (opts.path.to_owned() + &path).replace("\\", "/");
        while file_path.contains("//") {
            file_path = file_path.replace("//", "/");
        }
        let is_head = res.method == "HEAD";
        
        
        let mut rendered = false;
        let entry = GetByPath::new(&file_path);
        if entry.is_file {
            rendered = res.send_file(&entry.path, is_head) == 200;
        } else if opts.directory_listing && entry.is_directory {
            rendered = res.directory_listing(&entry.path, is_head) == 200;
        }
        if !rendered {
            let msg = "404 - file not found";
            res.set_header("Content-length", msg.len().to_string().as_str());
            res.set_status(404);
            res.write_string(msg);
            res.end();
        }
    }
}
