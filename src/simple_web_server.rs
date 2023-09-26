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
        println!("Request: {} {}", res.method, res.path);
        res.set_header("Connection", "keep-alive");
        res.set_header("Accept-ranges", "bytes");
        
        if opts.cors {
            res.set_header("access-control-allow-origin", "*");
            res.set_header("access-control-allow-methods", "GET, POST, PUT, DELETE");
            res.set_header("access-control-max-age", "120");
        }
        
        if res.method == "GET" || res.method == "HEAD" {
            SimpleWebServer::get(res, opts);
        } else if res.method == "PUT" {
            SimpleWebServer::put(res, opts);
        } else if res.method == "DELETE" {
            SimpleWebServer::delete(res, opts);
        } else {
            res.set_header("Content-length", "0");
            res.set_status(501);
            res.end();
        }
    }
    fn delete(mut res:Request, opts: Settings) {
        if !opts.delete {
            res.set_header("Content-length", "0");
            res.set_status(400);
            res.end();
            return;
        }
        let path = res.path.clone();
        let mut file_path = (opts.path.to_owned() + &path).replace("\\", "/");
        while file_path.contains("//") {
            file_path = file_path.replace("//", "/");
        }
        let entry = GetByPath::new(&file_path);
        if entry.error || entry.is_directory {
            res.set_header("Content-length", "0");
            res.set_status(404);
            res.end();
            return;
        }
        match std::fs::remove_file(&file_path) {
            Ok(_) => {
                res.set_header("Content-length", "0");
                res.set_status(200);
                res.end();
            }
            Err(_) => {
                res.set_header("Content-length", "0");
                res.set_status(500);
                res.end();
            }
        }
    }
    fn put(mut res:Request, opts: Settings) {
        if !opts.upload {
            res.set_header("Content-length", "0");
            res.set_status(400);
            res.end();
            return;
        }
        let path = res.path.clone();
        let mut file_path = (opts.path.to_owned() + &path).replace("\\", "/");
        while file_path.contains("//") {
            file_path = file_path.replace("//", "/");
        }
        let entry = GetByPath::new(&file_path);
        if (!entry.error && !opts.replace) || entry.is_directory {
            //file exists
            res.set_header("Content-length", "0");
            res.set_status(400);
            res.end();
            return;
        } else if !entry.error {
            match std::fs::remove_file(&file_path) {
                Ok(_) => {},
                Err(_) => {
                    res.set_header("Content-length", "0");
                    res.set_status(500);
                    res.end();
                    return;
                }
            }
        }
        if !res.write_to_file(&file_path) {
            res.set_header("Content-length", "0");
            res.set_status(500);
            res.end();
            return;
        }
        res.set_header("Content-length", "0");
        res.set_status(201);
        res.end();
    }
    fn get(mut res:Request, opts: Settings) {
        let path = res.path.clone();
        let mut file_path = (opts.path.to_owned() + &path).replace("\\", "/");
        while file_path.contains("//") {
            file_path = file_path.replace("//", "/");
        }
        let is_head = res.method == "HEAD";
        
        if opts.exclude_dot_html && res.origpath.ends_with(".html") || res.origpath.ends_with(".htm") {
            let mut new_path = res.origpath.clone();
            let new_length = new_path.len() - if res.origpath.ends_with(".html") { 5 } else { 4 };
            new_path.truncate(new_length);
            res.set_header("location", &new_path);
            res.set_status(307);
            res.end();
            return;
        }
        
        if opts.exclude_dot_html && res.origpath != "/" && !res.origpath.ends_with("/") {
            let entry = GetByPath::new(&(file_path.clone()+".html"));
            if !entry.error && entry.is_file {
                res.set_header("content-type", "text/html; charset=utf-8");
                if res.send_file(&entry.path, is_head) == 200 {
                    return;
                }
            }
            let entry2 = GetByPath::new(&(file_path.clone()+".htm"));
            if !entry2.error && entry2.is_file {
                res.set_header("content-type", "text/html; charset=utf-8");
                if res.send_file(&entry2.path, is_head) == 200 {
                    return;
                }
            }
        }
        
        
        let mut rendered = false;
        let entry = GetByPath::new(&file_path);
        if entry.is_file && res.origpath != "/" && res.origpath.ends_with("/") {
            res.set_header("Content-length", "0");
            let mut path = res.origpath.clone();
            path.pop();
            res.set_header("location", &path);
            res.set_status(301);
            res.end();
            return;
        }
        if entry.is_directory && !res.origpath.ends_with("/") {
            res.set_header("Content-length", "0");
            let path = res.origpath.clone();
            res.set_header("location", &(path+"/"));
            res.set_status(301);
            res.end();
            return;
        }
        if opts.index && entry.is_directory {
            if let Ok(paths) = std::fs::read_dir(file_path.clone()) {
                for path in paths {
                    let file = path.unwrap().path().display().to_string();
                    let name = file.split("/").last().unwrap_or("");
                    if name == "index.html" || name == "index.htm" {
                        res.set_header("content-type", "text/html; charset=utf-8");
                        if res.send_file(&(file_path.clone()+name), is_head) == 200 {
                            return;
                        }
                    } else if name == "index.xhtml" || name == "index.xhtm" {
                        res.set_header("content-type", "application/xhtml+xml; charset=utf-8");
                        if res.send_file(&(file_path.clone()+name), is_head) == 200 {
                            return;
                        }
                    }
                }
            }
        }
        
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
