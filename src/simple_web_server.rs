use crate::server::Server;
use crate::server::Settings;
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
        let path = res.path.clone();
        let mut file_path = (opts.path.to_owned() + &path).replace("\\", "/");
        while file_path.contains("//") {
            file_path = file_path.replace("//", "/");
        }
        //if this needs to be replaced - needs to be re-directed
        
        res.write_string(&file_path);
        res.end();
    }
}
