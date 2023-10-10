use std::net::TcpListener;
use std::thread;
use std::io::{ Read, Write, SeekFrom, Seek };
use std::fs;
use std::fs::File;
use std::str;
use std::time::Duration;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

static DIRECTORY_LISTING: &str = include_str!("directory-listing-template.html");

pub mod file_system;

pub mod mime;
use crate::server::mime::get_mime_type;

pub mod httpcodes;
use crate::server::httpcodes::get_http_message;

mod socket;
use crate::server::socket::Socket;

extern crate openssl;
use openssl::ssl::{SslMethod, SslAcceptor};
use openssl::rsa::Rsa;
use openssl::x509::X509Name;
use openssl::x509::X509;
use openssl::pkey::PKey;
use openssl::asn1::Asn1Time;
use openssl::asn1::Asn1Integer;
use openssl::x509::X509Builder;
use openssl::bn::BigNum;

fn to_acceptor(cert_str: &str, key_str: &str) -> SslAcceptor {
    
    let cert_str = cert_str
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join("\n");

    let key_str = key_str
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join("\n");

    let cert = X509::from_pem(cert_str.as_bytes()).unwrap();
    let key = Rsa::private_key_from_pem(key_str.as_bytes()).unwrap();
    
    let pkey = PKey::from_rsa(key).unwrap();
    
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key(&pkey).unwrap();
    builder.set_certificate(&cert).unwrap();
    
    builder.build()
}

pub fn generate_dummy_cert_and_key() -> Result<(String, String), openssl::error::ErrorStack> {
    let rsa = Rsa::generate(2048)?;
    let private_key = PKey::from_rsa(rsa)?;

    let mut x509_builder = X509Builder::new()?;
    x509_builder.set_version(2)?;
    
    let mut subject = X509Name::builder()?;
    subject.append_entry_by_text("commonName", "cn")?;
    subject.append_entry_by_text("countryName", "US")?;
    subject.append_entry_by_text("ST", "test-st")?;
    subject.append_entry_by_text("localityName", "Simple Web Server")?;
    subject.append_entry_by_text("organizationName", "Simple Web Server")?;
    subject.append_entry_by_text("OU", "SWS")?;
    subject.append_entry_by_text("CN", "127.0.0.1")?;
    let subject_name = subject.build();
    x509_builder.set_subject_name(&subject_name)?;

    x509_builder.set_issuer_name(&subject_name)?;

    let serial_number_bn = BigNum::from_u32(1).unwrap();

    let serial_number = Asn1Integer::from_bn(&serial_number_bn)?;
    x509_builder.set_serial_number(&serial_number)?;
    x509_builder.set_serial_number(&serial_number)?;

    let not_before = Asn1Time::days_from_now(0)?;
    let not_after = Asn1Time::days_from_now(365)?;
    x509_builder.set_not_before(&not_before)?;
    x509_builder.set_not_after(&not_after)?;

    x509_builder.set_pubkey(&private_key)?;

    x509_builder.sign(&private_key, openssl::hash::MessageDigest::sha256())?;
    
    let x509_cert = x509_builder.build();
    
    let cert = String::from_utf8_lossy(&x509_cert.to_pem()?).to_string();
    let key = String::from_utf8_lossy(&private_key.private_key_to_pem_pkcs8()?).to_string();

    Ok((cert, key))
}


#[derive(Copy, Clone)]
pub struct Settings<'a> {
    pub port: i32,
    pub path: &'a str,
    pub local_network: bool,
    pub spa: bool,
    pub rewrite_to: &'a str,
    pub directory_listing: bool,
    pub exclude_dot_html: bool,
    pub ipv6: bool,
    pub hidden_dot_files: bool,
    pub cors: bool,
    pub upload: bool,
    pub replace: bool,
    pub delete: bool,
    pub hidden_dot_files_directory_listing: bool,
    pub custom401: &'a str,
    pub custom403: &'a str,
    pub custom404: &'a str,
    pub custom500: &'a str,
    pub http_auth: bool,
    pub http_auth_username: &'a str,
    pub http_auth_password: &'a str,
    pub index: bool,
    pub https: bool,
    pub https_cert: &'a str,
    pub https_key: &'a str,
}

#[allow(dead_code)]
pub fn url_decode(input: &str) -> String {
    let mut decoded = String::new();
    let mut bytes = input.bytes();
    let mut utf8_buffer = Vec::new();

    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            if let (Some(hex1), Some(hex2)) = (bytes.next(), bytes.next()) {
                let hex_chars = [hex1, hex2];
                let hex_string: String = hex_chars.iter().map(|&x| x as char).collect();
                if let Ok(byte) = u8::from_str_radix(&hex_string, 16) {
                    utf8_buffer.push(byte);
                } else {
                    decoded.push('%');
                    decoded.push(hex1 as char);
                    decoded.push(hex2 as char);
                }
            } else {
                decoded.push('%');
            }
        } else {
            if !utf8_buffer.is_empty() {
                if let Ok(utf8_str) = str::from_utf8(&utf8_buffer) {
                    decoded.push_str(utf8_str);
                } else {
                    for b in &utf8_buffer {
                        decoded.push(*b as char);
                    }
                }
                utf8_buffer.clear();
            }
            decoded.push(byte as char);
        }
    }
    if !utf8_buffer.is_empty() {
        if let Ok(utf8_str) = str::from_utf8(&utf8_buffer) {
            decoded.push_str(utf8_str);
        } else {
            for b in &utf8_buffer {
                decoded.push(*b as char);
            }
        }
    }
    decoded
}

fn is_hidden(path: String) -> bool {
    let components: Vec<&str> = path.split('/').collect();
    for component in components.iter() {
        if component.starts_with('.') && component != &"." && component != &".." {
            return true;
        }
    }
    false
}

#[allow(dead_code)]
fn relative_path(cur_path: &str, req_path: &str) -> String {
    let mut end_with_slash = false;
    if req_path.ends_with('/') {
        end_with_slash = true;
    }
    
    let mut split1: Vec<&str> = cur_path.split('/').collect();
    let split2: Vec<&str> = req_path.split('/').collect();
    
    for w in split2.iter() {
        match *w {
            "" | "." => { /* . means current directory. Leave this here for spacing */ }
            ".." => {
                if !split1.is_empty() {
                    split1.pop();
                }
            }
            _ => {
                split1.push(w);
            }
        }
    }
    
    let mut new_path = split1.join("/");
    new_path = new_path.replace("//", "/");
    
    if !new_path.starts_with('/') {
        new_path = format!("/{}", new_path);
    }
    
    if end_with_slash && !new_path.ends_with('/') {
        new_path.push('/');
    }
    
    new_path
}

#[allow(dead_code)]
fn strip_off_file(orig_path: &str) -> String {
    if orig_path == "/" {
        return "/".to_string();
    }
    
    let last_slash_idx = orig_path.rfind('/').unwrap_or(0);
    orig_path[0..last_slash_idx].to_string()
}

#[allow(dead_code)]
#[allow(unused_assignments)]
struct Header {
    name: String,
    value: String
}
#[allow(dead_code)]
#[allow(unused_assignments)]
impl Header {
    pub fn new(header:&str, value:&str) -> Header {
        Header {
            name: header.to_string(),
            value: value.to_string()
        }
    }
}

#[allow(dead_code)]
#[allow(unused_assignments)]
pub struct Request<'a> {
    pub path: String,
    pub origpath: String,
    pub method: String,
    stream: &'a mut Socket,
    headers: Vec<Header>,
    out_headers: Vec<Header>,
    status_code: i32,
    status_message: String,
    headers_written: bool,
    length: usize,
    consumed: usize,
    finished: bool,
    connection_closed: bool
}

#[allow(dead_code)]
#[allow(unused_assignments)]
impl Request<'_> {
    pub fn new(stream:&mut Socket, head:String) -> Request {
        let lines = head.split("\r\n").collect::<Vec<_>>();
        let parts = lines[0].split(' ').collect::<Vec<_>>();
        let mut headers = Vec::new();
        let mut length = 0;
        for (i, line) in lines.iter().enumerate() {
            if i == 0 { continue; };
            let header_split = line.split(": ").collect::<Vec<_>>();
            if header_split.len() < 2 { continue; };
            headers.push(Header::new(header_split[0], header_split[1]));
            if header_split[0].to_lowercase() == "content-length" {
                match header_split[1].parse::<usize>() {
                    Ok(num) => {
                        length = num;
                    },
                    Err(_e) => {/*nada*/},
                }
            }
        }
        let path = relative_path("", &url_decode(parts[1].splitn(2, '?').collect::<Vec<_>>()[0]));
        let origpath = relative_path("", parts[1].splitn(2, '?').collect::<Vec<_>>()[0]);
        //todo, parse url arguments
        Request {
            method: parts[0].to_string(),
            path,
            origpath,
            stream,
            headers,
            out_headers: Vec::new(),
            status_code: 200,
            status_message: String::from("OK"),
            headers_written: false,
            length,
            consumed: 0,
            finished: false,
            connection_closed: false
        }
    }
    pub fn read(&mut self, bytes:usize) -> Result<Vec<u8>, bool> {
        let mut bytes = bytes;
        if self.consumed + bytes > self.length || bytes == 0 {
            //Consume the whole/rest of the body
            bytes = self.length - self.consumed;
        }
        if bytes == 0 {
            return Ok(b"".to_vec());
        }
        let mut buffer = vec![0; bytes];
        match self.stream.read(&mut buffer) {
            Ok(bytes_read) => {
                if buffer.len() > bytes_read {
                    //todo: exposed functions should be exact
                    buffer.truncate(bytes_read);
                }
                //println!("{} bytes read", bytes_read);
                self.consumed += bytes_read;
                Ok(buffer)
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
                Err(false)
            }
            Err(_) => {
                println!("read error");
                Err(true)
            }
        }
    }
    //Will truncate the file
    pub fn write_to_file(&mut self, path: &str) -> bool {
        let read_chunk_size = 1024 * 1024 * 4;
        let Ok(mut file) = File::create(path) else {
            return false;
        };
        let mut done = false;
        while !done {
            match self.read(read_chunk_size) {
                Ok(read) => {
                    done = read == b"";
                    match file.write(&read) {
                        Ok(_) => {},
                        Err(_e) => {
                            //uhh. what do we do here...
                            return false;
                        }
                    }
                }
                Err(fatal) => {
                    if fatal { return false; };
                }
            }
        }
        true
    }
    fn consume_body(&mut self) {
        let read_chunk_size = 1024 * 1024 * 4;
        while self.consumed != self.length {
            match self.read(read_chunk_size) {
                Ok(_) => {},
                Err(fatal) => {
                    if fatal { return; };
                }
            }
        }
    }
    pub fn read_string(&mut self, bytes:usize) -> String {
        let read = self.read(bytes).unwrap_or("".into());
        let msg = &String::from_utf8_lossy(&read[..read.len()]);
        msg.to_string()
    }
    fn send_headers(&mut self) {
        if self.headers_written {
            println!("Headers already sent!");
            return;
        }
        if !self.header_exists("Content-Length") {
            self.set_header("Transfer-Encoding", "Chunked");
        }
        //TODO - send date header
        let mut header = format!("HTTP/1.1 {} {}", self.status_code, self.status_message);
        for value in self.out_headers.iter() {
            header += &format!("\r\n{}:{}", value.name, value.value);
        }
        header += "\r\n\r\n";
        self.write_to_stream(header.as_bytes());
        self.headers_written = true;
    }
    fn write_to_stream(&mut self, data:&[u8]) -> bool {
        if self.connection_closed { return false; };
        let mut rv = true;
        match self.stream.write(data) {
            Ok(_e) => {
                rv = true;
            },
            Err(_e) => {
                rv = false;
                self.connection_closed = true;
            },
        };
        rv
    }
    pub fn write(&mut self, data:&[u8]) {
        if !self.headers_written { self.send_headers(); };
        let chunked = self.header_value_equals("Transfer-Encoding", "Chunked");
        if chunked {
            self.write_to_stream((format!("{:x}", data.len())+"\r\n").as_bytes());
        }
        self.write_to_stream(data);
        if chunked {
            self.write_to_stream("\r\n".as_bytes());
        }
    }
    fn format_header(&self, header:&str) -> String {
        let binding = header.to_string();
        let mut parts: Vec<String> = binding.split('-').map(|s| s.to_string()).collect();
        for part in parts.iter_mut() {
            *part = part.to_lowercase();
            let capitalized_char = part.chars().next().unwrap().to_uppercase().next().unwrap();
            let mut result = String::with_capacity(part.len());
            result.push(capitalized_char);
            result.push_str(&part[1..]);
            *part = result;
        }
        parts.join("-").to_string()
    }
    pub fn write_string(&mut self, data:&str) {
        self.write(data.to_string().as_bytes());
    }
    pub fn get_header(&mut self, header:&str) -> String {
        let head = self.format_header(header).to_string();
        for key in self.headers.iter() {
            if key.name == head {
                return key.value.as_str().to_string();
            }
        }
        String::new()
    }
    pub fn header_value_equals(&mut self, header:&str, value:&str) -> bool {
        let head = header.to_string();
        let val = value.to_string();
        for key in self.out_headers.iter() {
            if key.name == head {
                return key.value.to_lowercase() == val.to_lowercase();
            }
        }
        false
    }
    pub fn header_exists(&mut self, header:&str) -> bool {
        let head = header.to_string();
        for key in self.out_headers.iter() {
            if key.name == head {
                return true;
            }
        }
        false
    }
    pub fn set_header(&mut self, header:&str, value:&str) {
        // Are these header names and values valid?
        let new_header = Header::new(self.format_header(header).as_str(), value);
        for key in self.out_headers.iter_mut() {
            if key.name == new_header.name {
                *key = new_header;
                return;
            }
        }
        
        self.out_headers.push(new_header);
    }
    pub fn set_status(&mut self, code:i32) {
        self.status_code = code;
        self.status_message = get_http_message(code);
    }
    pub fn end(&mut self) {
        if self.finished { return; };
        self.consume_body();
        if !self.headers_written { self.send_headers(); };
        self.finished = true;
        let chunked = self.header_value_equals("Transfer-Encoding", "Chunked");
        if chunked {
            self.write_to_stream("0\r\n\r\n".as_bytes());
        } else {
            self.write_to_stream("\r\n\r\n".as_bytes());
        }
    }
    //The directory_listing and send_file functions will return either 404, 500, or 200
    pub fn directory_listing(&mut self, path:&str, no_body:bool, dot_files:bool) -> i32 {
        if self.headers_written {
            println!("Headers must not yet be sent when using send_file");
            return 500;
        }
        self.set_header("content-type", "text/html; charset=utf-8");
        let Ok(paths) = fs::read_dir(path) else {
            return 404;
        };
        let mut to_send = String::from("<!DOCTYPE html>\n<html dir=\"ltr\" lang=\"en\n<head><meta charset=\"utf-8\"><meta name=\"google\" value=\"notranslate\"><title id=\"title\"></title>\n</head>\n<body><div id=\"staticListing\"><style>li.directory {background:#aab}</style><a href=\"../\">parent</a><ul>");
        let mut js_listing = String::new();
        for path in paths {
            let file = path.unwrap();
            let name = file.path().display().to_string();
            if !dot_files && is_hidden(name.clone()) { continue; };
            let file_name = name.split('/').last().unwrap_or("");
            if file.path().is_dir() {
                to_send += &format!("<li class=\"directory\"><a href=\"{}/\">{}</a></li>", file_name, file_name);
            } else {
                to_send += &format!("<li><a href=\"{}/\">{}</a></li>", file_name, file_name);
            }
            
            let rawname = name.split('/').last().unwrap_or("").replace('"', "\\\"");
            let is_dir = if file.path().is_dir() { "true" } else { "false" };
            let modified = 0;
            let modifiedstr = "";
            let filesize = 0;
            let filesizestr = "";
            
            js_listing += &format!("<script>addRow(\"{}\", \"{}\", {}, \"{}\", \"{}\", \"{}\", \"{}\");</script>", rawname, rawname, is_dir, filesize, filesizestr, modified, modifiedstr);
        }
        to_send += &format!("</ul></div><div style=\"display: none;\" id=\"niceListing\">\n{}", DIRECTORY_LISTING);
        
        if self.origpath != "/" {
            to_send += "<script>onHasParentDirectory();</script>";
        }
        to_send += &format!("<script>start(\"{}\")</script>", self.path.replace('"', "\\\""));
        
        to_send += &js_listing;
        
        to_send += "</div></body></html>";
        
        let bytes = to_send.as_bytes();
        self.set_header("content-length", &bytes.len().to_string());
        if !no_body {
            self.write(bytes);
        }
        self.end();
        
        200
    }
    pub fn send_file(&mut self, path:&str, no_body:bool) -> i32 {
        if self.headers_written {
            println!("Headers must not yet be sent when using send_file");
            return 500;
        }
        //println!("Rendering file at {}", path);
        let read_chunk_size : usize = 1024 * 1024 * 8;
        let Ok(mut file) = File::open(path) else {
            return 404;
        };
        let ext = path.split('.').last().unwrap();
        self.set_header("content-type", get_mime_type(ext));
        let size : usize = file.metadata().unwrap().len().try_into().unwrap();
        let mut written : usize = 0;
        
        let mut file_offset : usize = 0;
        let mut file_end_offset : usize = size - 1;
        let mut content_length : usize = size;
        let mut code = 200;
        let range_header = self.get_header("Range");
        //println!("{}", self.get_header("Range"));
        if range_header != String::new() {
            //println!("Range Request");
            let range = range_header.split('=').collect::<Vec<_>>()[1].trim();
            let rparts = range.split('-').collect::<Vec<_>>();
            match rparts[0].parse::<usize>() {
                Ok(num) => {
                    file_offset = num;
                },
                Err(_e) => {/*nada*/},
            }
            //println!("{} {}", range_header, rparts[1].len());
            if rparts[1].is_empty() {
                //file_end_offset = size - 1;
                content_length = size - file_offset;
                
                self.set_header("content-range", &format!("bytes {}-{}/{}", file_offset, size-1, size));
                code = if file_offset == 0 { 200 } else { 206 };
            } else {
                match rparts[1].parse::<usize>() {
                    Ok(num) => {
                        file_end_offset = num;
                    },
                    Err(_e) => {/*nada*/},
                }
                content_length = file_end_offset - file_offset + 1;
                self.set_header("content-range", &format!("bytes {}-{}/{}", file_offset, file_end_offset, size));
                code = 206;
            }
        }
        
        self.set_header("content-length", &content_length.to_string());
        self.set_status(code);
        if no_body {
            drop(file);
            self.end();
            return 200;
        }
        let Ok(_) = file.seek(SeekFrom::Start(file_offset.try_into().unwrap())) else { return 500; };
        while written < content_length {
            if self.connection_closed { break; };
            let chunk_size : usize = if content_length-written > read_chunk_size { read_chunk_size } else { content_length-written };
            if chunk_size == 0 { break; };
            let mut buffer = vec![0; chunk_size];
            let Ok(_) = file.read(&mut buffer) else { todo!() };
            self.write(&buffer);
            written += chunk_size
        }
        drop(file);
        self.end();
        200
    }
}

fn read_header(stream:&mut Socket, on_request:fn(Request, Settings), user_data: Settings, stopped_clone: &Arc<AtomicBool>) -> bool {
    let mut buffer = [0; 1];
    let mut request = String::new();
    
    loop {
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    return false;
                }
                match &String::from_utf8(buffer[..bytes_read].to_vec()) {
                    Ok(s) => {
                        request += s;
                    },
                    //an https request to an http server? Either way, we cant do anything with the data
                    Err(_) => { return false; },
                }
                
                if request.ends_with("\r\n\r\n") {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                let value = stopped_clone.load(Ordering::Relaxed);
                if value { break; }
                thread::sleep(Duration::from_millis(10));
            }
            Err(_) => {
                break;
            },
        }
    }
    if request.is_empty() {
        return false;
    }
    (on_request)(Request::new(stream, request), user_data);
    true
}

#[allow(dead_code)]
pub struct Server {
    opts: Settings<'static>,
    sender: Option<mpsc::Sender<String>>,
    receiver: Arc<Mutex<mpsc::Receiver<String>>>,
    running: bool,
    on_request: fn(Request, Settings)
}

#[allow(dead_code)]
impl Server {
    pub fn new(opts: Settings<'static>, on_request: fn(Request, Settings)) -> Server {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        Server {
            opts,
            receiver,
            sender: Some(sender),
            running: false,
            on_request
        }
    }
    pub fn start(&mut self) -> bool {
        let receiver = self.receiver.clone();
        let opts = self.opts;
        let host = if opts.local_network {
            if opts.ipv6 { "::" } else { "0.0.0.0" }
        } else if opts.ipv6 { "::1" } else { "127.0.0.1" };
        let port = opts.port;
        let on_request = self.on_request;
        match TcpListener::bind(format!("{}:{}", host, port)) {
            Ok(listener) => {
                match listener.set_nonblocking(true) {
                    Ok(_) => {},
                    Err(_) => { return false; },
                }
                thread::spawn(move || {
                    let stopped: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
                    
                    println!("Server started on http{}://{}:{}/", if opts.https { "s" } else { "" }, host, port);
                    for stream in listener.incoming() {
                        match stream {
                            Ok(stream) => {
                                let stopped_clone = Arc::clone(&stopped);
                                if opts.https {
                                    let _ = listener.set_nonblocking(false);
                                    let acceptor = to_acceptor(opts.https_cert, opts.https_key);
                                    thread::spawn(move || {
                                        match acceptor.accept(stream) {
                                            Ok(stream) => {
                                                let _ = stream.get_ref().set_nonblocking(true);
                                                let mut socket = Socket::new(None, Some(stream));
                                                while read_header(&mut socket, on_request, opts, &stopped_clone) {
                                                    // keep alive
                                                }
                                                socket.drop();
                                            }
                                            Err(ref _e) => {
                                                //99% of the time this is an ssl handshake error. We should be able to safely ignore this.
                                            }
                                        }
                                        drop(acceptor);
                                        drop(stopped_clone);
                                    });
                                } else {
                                    thread::spawn(move || {
                                        let mut socket = Socket::new(Some(stream), None);
                                        while read_header(&mut socket, on_request, opts, &stopped_clone) {
                                            // keep alive
                                        }
                                        socket.drop();
                                        drop(stopped_clone);
                                    });
                                }
                                
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                let message = receiver.lock().unwrap().try_recv();
                                if let Ok(job) = message {
                                    if job == *"kill" {
                                        stopped.store(true, Ordering::Relaxed);
                                        break;
                                    }
                                }
                                thread::sleep(Duration::from_millis(10));
                            }
                            Err(e) => panic!("encountered IO error: {}", e),
                        }
                    }
                    drop(listener);
                });
            },
            Err(_) => {
                println!("Failed to listen on http://{}:{}/", host, port);
                return false;
            },
        }
        self.running = true;
        true
    }
    pub fn terminate(&mut self) {
        if !self.running { return; };
        println!("Killing server");
        self.running = false;
        self.sender.as_ref().unwrap().send(String::from("kill")).unwrap();
    }
}
