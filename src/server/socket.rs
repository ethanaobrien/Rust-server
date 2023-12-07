use std::net::TcpStream;
use openssl::ssl::SslStream;
use std::io;
use std::io::{Read, Write};
use std::io::{Error, ErrorKind};

pub struct Socket {
    stream: Option<TcpStream>,
    ssl_stream: Option<SslStream<TcpStream>>
}

impl Socket {
    pub fn new(stream: Option<TcpStream>, ssl: Option<SslStream<TcpStream>>) -> Socket {
        Socket {
            stream,
            ssl_stream: ssl
        }
    }
    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.ssl_stream {
            Some(ref mut ssl_stream) => {
                ssl_stream.read(buf)
            }
            None => {
                match self.stream {
                    Some(ref mut stream) => {
                        stream.read(buf)
                    }
                    None => {
                        println!("Error getting socket type. This should not be possible!!");
                        Ok(0)
                    }
                }
            }
        }
    }
    pub fn write(&mut self, buf: &[u8]) -> io::Result<()> {
        match self.ssl_stream {
            Some(ref mut ssl_stream) => {
                ssl_stream.write_all(buf)
            }
            None => {
                match self.stream {
                    Some(ref mut stream) => {
                        stream.write_all(buf)
                    }
                    None => {
                        println!("Error getting socket type. This should not be possible!!");
                        Ok(())
                    }
                }
            }
        }
    }
    pub fn peek(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.ssl_stream {
            Some(ref mut ssl_stream) => {
                match ssl_stream.ssl_peek(buf) {
                    Ok(e) => {Ok(e)},
                    Err(_) => {Err(Error::new(ErrorKind::Other, "oh no!"))}
                }
            }
            None => {
                match self.stream {
                    Some(ref mut stream) => {
                        stream.peek(buf)
                    }
                    None => {
                        println!("Error getting socket type. This should not be possible!!");
                        Ok(0)
                    }
                }
            }
        }
    }
    pub fn drop(self) {
        drop(self.ssl_stream);
        drop(self.stream);
    }
}
