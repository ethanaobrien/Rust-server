use std::io::Write;
use std::net::TcpStream;
use std::net::TcpListener;
use std::thread;
use std::io::Read;

fn main() {
    create_server("127.0.0.1".to_string(), 8888);
}

fn write(mut stream:&TcpStream, data:&[u8], headers:&Vec<String>) {
    let mut header = "HTTP/1.1 200 OK\r\n".to_string();
    for value in headers {
        if value.len() == 0 { continue; };
        header += &(value.to_owned()+"\r\n");
    }
    header += "\r\n";
    stream.write(header.as_bytes()).unwrap();
    stream.write(data).unwrap();
}

struct Request {
    path: String,
    method: String,
    stream: TcpStream,
    headers: Vec<String>
}

impl Request {
    pub fn new(stream:TcpStream, method:String, path:String) -> Request {
        Request { method: method, path: path, stream: stream, headers: vec![String::new(); 126] }
    }
    pub fn write_string(&self, data:&str) {
        write(&self.stream, data.to_string().as_bytes(), &self.headers);
    }
    pub fn set_header(&mut self, header:&str, value:&str) {
        // Are these header names and values valid?
        // Are the headers being re-set?
        self.headers.push(header.to_owned()+": "+value);
    }
    pub fn end(self) {
        drop(self.stream);
    }
    
}

fn on_request(mut res:Request) {
    res.set_header("Connection", "close");
    res.set_header("Content-Type", "text/plain");
    res.write_string("Hello");
    res.end();
    
    //need accept-ranges, type, length, date, keep alive
    
}

fn create_server(host:String, port:i32) {
    let listener = TcpListener::bind(host.clone()+":"+&port.to_string()).unwrap();
    println!("Server started on http://{}:{}/", host, port);
    for stream in listener.incoming() {
        thread::spawn(|| {
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
            on_request(Request::new(stream, parts[0].to_string(), parts[1].to_string()));
        });
    }
}
