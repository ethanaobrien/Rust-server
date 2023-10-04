use crate::server::Server;
use crate::server::Settings;
use crate::server::file_system::GetByPath;
use crate::server::Request;
use crate::server::httpcodes::get_http_message;

pub struct SimpleWebServer {
    server: Server
}

const BASE_CHARS: [u8; 64] = [
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P',
    b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'+', b'/',
];

pub fn decode_base64(input: &[u8]) -> String {
	let mut output: Vec<u8> = Vec::new();
	for chunk in input.chunks(4) {

    	let a = decode_char(chunk[0]);
    	let b = decode_char(chunk[1]);
    	let c = decode_char(chunk[2]);
    	let d = decode_char(chunk[3]);

    	let dec1 = ((a << 2) | (b & 0x30) >> 4) as u8;
    	let dec2 = (((b & 0x0F) << 4) | (c & 0x3C) >> 2) as u8;
    	let dec3 = (((c & 0x03) << 6) | (d)) as u8;

    	output.push(dec1);
    	output.push(dec2);
    	output.push(dec3);
	}

	String::from_utf8(output).unwrap_or(String::new()).replace("\0", "")
}
fn decode_char(input: u8) -> u8 {
	BASE_CHARS.iter().position(|&c| c == input).unwrap_or(0) as u8
}

#[allow(dead_code)]
impl SimpleWebServer {
    fn log(msg: String) {
        println!("{}", msg);
    }
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
    fn validate_auth(auth: String, username: &str, password: &str) -> bool {
        if auth == "" { return false; };
        if !auth.to_lowercase().starts_with("basic ") { return false; };
        let base64_data = &auth[6..];
        let decoded_str = decode_base64(base64_data.as_bytes());
        if decoded_str == "" || decoded_str == ":" { return false; };
        if let Some(index) = decoded_str.find(':') {
            let auth_username = &decoded_str[0..index];
            let auth_password = &decoded_str[index + 1..];
            
            return auth_username == username && auth_password == password;
        }
        return false;
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
        
        if opts.http_auth {
            if !Self::validate_auth(res.get_header("authorization"), opts.http_auth_username, opts.http_auth_password) {
                Self::error(res, opts, "", 401);
                return;
            }
        }
        
        let mut rewrite_to = "";
        if opts.spa && !res.path.contains(".") {
            rewrite_to = if opts.rewrite_to != "" { opts.rewrite_to } else { "/index.html" };
        }
        
        if res.method == "GET" || res.method == "HEAD" {
            Self::get(res, opts, rewrite_to);
        } else if res.method == "PUT" {
            Self::put(res, opts);
        } else if res.method == "DELETE" {
            Self::delete(res, opts);
        } else {
            Self::error(res, opts, "", 501);
        }
    }
    fn error(mut res:Request, opts: Settings, msg: &str, code: i32) {
        if code == 401 {
            res.set_header("WWW-Authenticate", "Basic realm=\"SimpleWebServer\", charset=\"UTF-8\"");
        }
        res.set_status(code);
        if ((code == 401 && opts.custom401 != "") ||
           (code == 403 && opts.custom403 != "") ||
           (code == 404 && opts.custom404 != "") ||
           (code == 500 && opts.custom500 != "")) &&
           msg != "NONOTUSECUSTOM" {
            let path = if code == 401 {opts.custom401} else if code == 403 {opts.custom403} else if code == 404 {opts.custom404} else if code == 500 {opts.custom500} else {""};
            let file_path = Self::from_relative(opts, path.clone().to_string());
            let entry = GetByPath::new(&file_path);
            if !entry.error && entry.is_file {
                if entry.is_hidden() && !opts.hidden_dot_files {
                    Self::error(res, opts, if code == 404 { "NONOTUSECUSTOM" } else { "" }, 404);
                    return;
                }
                if res.send_file(&entry.path, res.method == "HEAD") == 200 {
                    return;
                }
            } else {
                Self::log(format!("Failed to read from custom {} path (\"{}\")", code, file_path));
            }
        }
        res.set_header("content-type", "text/html; charset=utf-8");
        
        let def_msg = format!("<h1>{} - {}</h1>\n\n<p>{}</p>", code, get_http_message(code), msg);
        let default_msg = def_msg.as_bytes();
        let size = default_msg.len();
        res.set_header("Content-length", &size.to_string());
        if res.method != "HEAD" {
            res.write(default_msg);
        }
        res.end();
    }
    fn from_relative(opts: Settings, path: String) -> String {
        let mut file_path = format!("{}{}", opts.path.to_owned(), path).replace("\\", "/");
        while file_path.contains("//") {
            file_path = file_path.replace("//", "/");
        }
        return file_path;
    }
    fn delete(mut res:Request, opts: Settings) {
        if !opts.delete {
            res.set_header("Content-length", "0");
            res.set_status(400);
            res.end();
            return;
        }
        let file_path = Self::from_relative(opts, res.path.clone());
        let entry = GetByPath::new(&file_path);
        if entry.error || entry.is_directory {
            Self::error(res, opts, "", 404);
            return;
        }
        match std::fs::remove_file(&file_path) {
            Ok(_) => {
                res.set_header("Content-length", "0");
                res.set_status(200);
                res.end();
            }
            Err(_) => {
                Self::error(res, opts, "", 500);
            }
        }
    }
    fn put(mut res:Request, opts: Settings) {
        if !opts.upload {
            Self::error(res, opts, "", 400);
            return;
        }
        let file_path = Self::from_relative(opts, res.path.clone());
        let entry = GetByPath::new(&file_path);
        if (!entry.error && !opts.replace) || entry.is_directory {
            //file exists
            Self::error(res, opts, "", 400);
            return;
        } else if !entry.error {
            match std::fs::remove_file(&file_path) {
                Ok(_) => {},
                Err(_) => {
                    Self::error(res, opts, "", 500);
                    return;
                }
            }
        }
        if !res.write_to_file(&file_path) {
            Self::error(res, opts, "", 500);
            return;
        }
        res.set_header("Content-length", "0");
        res.set_status(201);
        res.end();
    }
    fn get(mut res:Request, opts: Settings, rewrite_to: &str) {
        let path = if rewrite_to == "" { res.path.clone() } else { rewrite_to.to_string() };
        let file_path = Self::from_relative(opts, path);
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
                if entry.is_hidden() && !opts.hidden_dot_files {
                    Self::error(res, opts, "", 404);
                    return;
                }
                res.set_header("content-type", "text/html; charset=utf-8");
                if res.send_file(&entry.path, is_head) == 200 {
                    return;
                }
            }
            let entry2 = GetByPath::new(&(file_path.clone()+".htm"));
            if !entry2.error && entry2.is_file {
                if entry.is_hidden() && !opts.hidden_dot_files {
                    Self::error(res, opts, "", 404);
                    return;
                }
                res.set_header("content-type", "text/html; charset=utf-8");
                if res.send_file(&entry2.path, is_head) == 200 {
                    return;
                }
            }
        }
        
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
                        if entry.is_hidden() && !opts.hidden_dot_files {
                            Self::error(res, opts, "", 404);
                            return;
                        }
                        res.set_header("content-type", "text/html; charset=utf-8");
                        if res.send_file(&(file_path.clone()+name), is_head) == 200 {
                            return;
                        }
                    } else if name == "index.xhtml" || name == "index.xhtm" {
                        if entry.is_hidden() && !opts.hidden_dot_files {
                            Self::error(res, opts, "", 404);
                            return;
                        }
                        res.set_header("content-type", "application/xhtml+xml; charset=utf-8");
                        if res.send_file(&(file_path.clone()+name), is_head) == 200 {
                            return;
                        }
                    }
                }
            }
        }
        
        
        let mut rendered = false;
        if entry.is_hidden() && !opts.hidden_dot_files {
            Self::error(res, opts, "", 404);
            return;//rust will complain about a "moved value" so just return.
        } else if entry.is_file {
            rendered = res.send_file(&entry.path, is_head) == 200;
        } else if opts.directory_listing && entry.is_directory {
            rendered = res.directory_listing(&entry.path, is_head, opts.hidden_dot_files_directory_listing) == 200;
        }
        if !rendered {
            Self::error(res, opts, "", 404);
        }
    }
}
