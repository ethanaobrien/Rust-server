use std::net::TcpStream;
use std::net::TcpListener;
use std::thread;
use std::io::{ Read, Write, SeekFrom, Seek };
use std::fs::File;
use std::str;

pub fn url_decode(input: &str) -> String {
    let mut decoded = String::new();
    let mut bytes = input.bytes();
    let mut utf8_buffer = Vec::new();

    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            if let (Some(hex1), Some(hex2)) = (bytes.next(), bytes.next()) {
                let hex_chars = vec![hex1, hex2];
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
    return decoded;
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
    pub method: String,
    stream: &'a TcpStream,
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
            consumed: 0,
            finished: false,
            connection_closed: false
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
        //TODO - send date header
        let mut header = ("HTTP/1.1 ".to_owned()+&self.status_code.to_string()+" "+self.status_message.as_str()).to_string();
        for value in self.out_headers.iter() {
            let key = value.name.to_owned()+": "+value.value.as_str();
            header += &("\r\n".to_owned()+key.as_str());
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
        return rv;
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
    pub fn end(&mut self) {
        if self.finished { return; };
        if !self.headers_written { self.send_headers(); };
        self.finished = true;
        let chunked = self.header_value_equals("Transfer-Encoding", "Chunked");
        if chunked {
            self.write_to_stream("0\r\n\r\n".as_bytes());
        } else {
            self.write_to_stream("\r\n\r\n".as_bytes());
        }
    }
    pub fn send_file(&mut self, path:&str) -> bool {
        if self.headers_written {
            println!("Headers must not yet be sent when using send_file");
            return false;
        }
        println!("Rendering file at {}", path);
        let read_chunk_size : usize = 1024 * 1024 * 8;
        let Ok(mut file) = File::open(path) else {
            return false;
        };
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
            let range = range_header.split("=").collect::<Vec<_>>()[1].trim();
            let rparts = range.split("-").collect::<Vec<_>>();
            match rparts[0].parse::<usize>() {
                Ok(num) => {
                    file_offset = num;
                },
                Err(_e) => {/*nada*/},
            }
            //println!("{} {}", range_header, rparts[1].len());
            if rparts[1].len() == 0 {
                //file_end_offset = size - 1;
                content_length = size - file_offset;
                self.set_header("content-range", &("bytes ".to_owned()+&file_offset.to_string()+"-"+&(size-1).to_string()+"/"+&size.to_string()));
                code = if file_offset == 0 { 200 } else { 206 };
            } else {
                match rparts[1].parse::<usize>() {
                    Ok(num) => {
                        file_end_offset = num;
                    },
                    Err(_e) => {/*nada*/},
                }
                content_length = file_end_offset - file_offset + 1;
                self.set_header("content-range", &("bytes ".to_owned()+&file_offset.to_string()+"-"+&file_end_offset.to_string()+"/"+&size.to_string()));
                code = 206;
            }
        }
        let Ok(_) = file.seek(SeekFrom::Start(file_offset.try_into().unwrap())) else { todo!() };
        
        self.set_header("content-length", &content_length.to_string());
        self.set_status(code, if code == 200 { "OK" } else { "Partial Content" });
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
        return true;
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
    if request.len() == 0 {
        return false;
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