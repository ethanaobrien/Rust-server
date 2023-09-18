use std::io::Write;
use std::net::TcpStream;
use std::net::TcpListener;
use std::thread;
use std::io::Read;


#[allow(dead_code)]
struct Header {
    name: String,
    value: String
}
#[allow(dead_code)]
impl Header {
    pub fn new(header:&str, value:&str) -> Header {
        Header {
            name: header.to_string(),
            value: value.to_string()
        }
    }
}

#[allow(dead_code)]
pub struct Request<'a> {
    pub path: String,
    pub method: String,
    stream: &'a TcpStream,
    headers: Vec<Header>,
    out_headers: Vec<Header>,
    status_code: i32,
    status_message: String,
    headers_written: bool,
    length: usize,
    consumed: usize
}

#[allow(dead_code)]
impl Request<'_> {
    pub fn new(stream:&TcpStream, head:String) -> Request {
        let lines = head.split("\r\n").collect::<Vec<_>>();
        let parts = lines[0].split(" ").collect::<Vec<_>>();
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
        Request {
            method: parts[0].to_string(),
            path: parts[1].to_string(),
            stream: stream,
            headers: headers,
            out_headers: Vec::new(),
            status_code: 200,
            status_message: String::from("OK"),
            headers_written: false,
            length: length,
            consumed: 0
        }
    }
    pub fn read(&mut self, bytes:usize) -> Vec<u8> {
        let mut bytes = bytes;
        if self.consumed + bytes > self.length || bytes == 0 {
            //Consume the whole/rest of the body
            bytes = self.length - self.consumed;
        }
        if bytes <= 0 {
            return b"".to_vec();
        }
        let mut buffer = vec![0; bytes];
        let Ok(bytes_read) = self.stream.read(&mut buffer) else {
            println!("Read error");
            return b"".to_vec();
        };
        //println!("{} bytes read", bytes_read);
        self.consumed += bytes_read;
        return buffer;
    }
    pub fn read_string(&mut self, bytes:usize) -> String {
        let read = self.read(bytes);
        let msg = &String::from_utf8_lossy(&read[..read.len()]);
        return msg.to_string();
    }
    fn send_headers(&mut self) {
        if self.headers_written {
            println!("Headers already sent!");
            return;
        }
        if !self.header_exists("Content-Length") {
            self.set_header("Transfer-Encoding", "Chunked");
        }
        let mut header = ("HTTP/1.1 ".to_owned()+&self.status_code.to_string()+" "+self.status_message.as_str()).to_string();
        for value in self.out_headers.iter() {
            let key = value.name.to_owned()+": "+value.value.as_str();
            header += &("\r\n".to_owned()+key.as_str());
        }
        header += "\r\n\r\n";
        self.stream.write(header.as_bytes()).unwrap();
        self.headers_written = true;
    }
    pub fn write(&mut self, data:&[u8]) {
        if !self.headers_written { self.send_headers(); };
        let chunked = self.header_value_equals("Transfer-Encoding", "Chunked");
        if chunked {
            self.stream.write((format!("{:x}", data.len())+"\r\n").as_bytes()).unwrap();
        }
        self.stream.write(data).unwrap();
        if chunked {
            self.stream.write("\r\n".as_bytes()).unwrap();
        }
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
    pub fn get_header(&mut self, header:&str) -> String {
        let head = self.format_header(header).to_string();
        for key in self.headers.iter() {
            if key.name == head {
                return key.value.as_str().to_string();
            }
        }
        return String::new();
    }
    pub fn header_value_equals(&mut self, header:&str, value:&str) -> bool {
        let head = header.to_string();
        let val = value.to_string();
        for key in self.out_headers.iter() {
            if key.name == head {
                return key.value.to_lowercase() == val.to_lowercase();
            }
        }
        return false;
    }
    pub fn header_exists(&mut self, header:&str) -> bool {
        let head = header.to_string();
        for key in self.out_headers.iter() {
            if key.name == head {
                return true;
            }
        }
        return false;
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
    pub fn set_status(&mut self, code:i32, msg:&str) {
        self.status_code = code;
        self.status_message = msg.to_string();
    }
    pub fn end(mut self) {
        if !self.headers_written { self.send_headers(); };
        let chunked = self.header_value_equals("Transfer-Encoding", "Chunked");
        if chunked {
            self.stream.write("0\r\n\r\n".as_bytes()).unwrap();
        } else {
            self.stream.write("\r\n\r\n".as_bytes()).unwrap();
        }
        //
    }
}

fn read_header(mut stream:&TcpStream, on_request:fn(res:Request)) -> bool {
    let mut buffer = [0; 1];
    let mut request = String::new();
    while let Ok(bytes_read) = stream.read(&mut buffer) {
        if bytes_read == 0 {
            return false;
        }
        request += &String::from_utf8_lossy(&buffer[..bytes_read]);
        if request.ends_with("\r\n\r\n") {
            break;
        }
    }
    (on_request)(Request::new(stream, request));
    return true;
}

pub fn create_server(host:&str, port:i32, on_request:fn(res:Request)) {
    let listener = TcpListener::bind(host.to_string().to_owned()+":"+&port.to_string()).unwrap();
    println!("Server started on http://{}:{}/", host, port);
    for stream in listener.incoming() {
        thread::spawn(move || {
            while read_header(stream.as_ref().unwrap(), on_request) {
                // keep alive
            }
            drop(stream);
        });
    }
}
