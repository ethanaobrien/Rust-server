use std::io::Write;
use std::net::TcpStream;
use std::net::TcpListener;
use std::thread;
use std::io::Read;

struct Header {
    name: String,
    value: String
}
impl Header {
    pub fn new(header:&str, value:&str) -> Header {
        Header {
            name: header.to_string(),
            value: value.to_string()
        }
    }
}

pub struct Request {
    path: String,
    method: String,
    stream: TcpStream,
    headers: Vec<Header>,
    status_code: i32,
    status_message: String
}

impl Request {
    pub fn new(stream:TcpStream, method:String, path:String) -> Request {
        Request {
            method: method,
            path: path,
            stream: stream,
            headers: Vec::new(),
            status_code: 200,
            status_message: String::from("OK")
        }
    }
    pub fn write(&mut self, data:&[u8]) {
        if !self.header_exists("Content-Length") {
            self.set_header("Content-Length", data.len().to_string().as_str());
        }
        let mut header = ("HTTP/1.1 ".to_owned()+&self.status_code.to_string()+" "+self.status_message.as_str()+"\r\n").to_string();
        for value in self.headers.iter() {
            let key = value.name.to_owned()+": "+value.value.as_str();
            header += &(key.to_owned()+"\r\n");
        }
        header += "\r\n";
        self.stream.write(header.as_bytes()).unwrap();
        self.stream.write(data).unwrap();
    }
    fn format_header(&self, header:&str) -> String {
        let binding = header.to_string();
        let mut parts: Vec<String> = binding.split("-").map(|s| s.to_string()).collect();
        for part in parts.iter_mut() {
            *part = part.to_lowercase();
            let capitalized_char = part.chars().next().unwrap().to_uppercase().next().unwrap();
            let mut result = String::with_capacity(part.len());
            result.push(capitalized_char);
            result.push_str(&part[1..]);
            *part = result;
        }
        return parts.join("-").to_string();
    }
    pub fn write_string(&mut self, data:&str) {
        self.write(data.to_string().as_bytes());
    }
    pub fn header_exists(&mut self, header:&str) -> bool {
        let head = header.to_string();
        for key in self.headers.iter() {
            if key.name == head {
                return true;
            }
        }
        return false;
    }
    pub fn set_header(&mut self, header:&str, value:&str) {
        // Are these header names and values valid?
        let new_header = Header::new(self.format_header(header).as_str(), value);
        for key in self.headers.iter_mut() {
            if key.name == new_header.name {
                *key = new_header;
                return;
            }
        }
        
        self.headers.push(new_header);
    }
    pub fn set_status(&mut self, code:i32, msg:&str) {
        self.status_code = code;
        self.status_message = msg.to_string();
    }
    pub fn end(self) {
        drop(self.stream);
    }
    
}

pub fn create_server(host:&str, port:i32, on_request:fn(res:Request)) {
    let listener = TcpListener::bind(host.to_string().to_owned()+":"+&port.to_string()).unwrap();
    println!("Server started on http://{}:{}/", host, port);
    for stream in listener.incoming() {
        thread::spawn(move || {
            let mut stream = stream.unwrap();
            let mut buffer = [0; 1];
            let mut request = String::new();
            while let Ok(bytes_read) = stream.read(&mut buffer) {
                if bytes_read == 0 {
                    break;
                }
                request += &String::from_utf8_lossy(&buffer[..bytes_read]);
                if request.ends_with("\r\n\r\n") {
                    break;
                }
            }
            let parts = request.split("\r\n").collect::<Vec<_>>()[0].split(" ").collect::<Vec<_>>();
            (on_request)(Request::new(stream, parts[0].to_string(), parts[1].to_string()));
        });
    }
}
