pub mod request;

use request::{HttpRequest, HttpResponse};

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Result, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

/**
 * # struct Server
 * exemple
 * ```
 * use hddp::Server;
 * fn main() -> std::io::Result<()> {
 *     let server = Server::new();
 *     server.listen("127.0.0.1:8080")?;
 *     Ok(())
 * }
 * ```
 */
#[derive(Clone)]
pub struct Server<'a> {
    paths: HashMap<&'a str, Vec<u8>>,
}

impl<'a> Default for Server<'a> {
    fn default() -> Self {
        let mut paths = HashMap::new();
        let file = match fs::read_to_string("pages/default/index.html") {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Failed to read the file: {}", e);
                "404".to_string()
            }
        };
        let response = HttpResponse::new(
            file.as_str(),
        );
        paths.insert("/", response.into_bytes());
        Server { paths }
    }
}

impl<'a> Server<'a> {
    fn handle_client(&self, mut stream: TcpStream) {
        let mut buffer: [u8; 1024] = [0; 1024];
        let bytes_read = match stream.read(&mut buffer) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Failed to read byte from stream: {}", e);
                return;
            }
        };
        let request = match HttpRequest::from(&buffer[..bytes_read]) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Failed to read byte from stream: {}", e);
                return;
            }
        };
        println!("{} {}", request.method, request.path);
        let file = match fs::read_to_string("pages/404/index.html") {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Failed to read the file: {}", e);
                "404".to_string()
            }
        };
        let mut response = &HttpResponse::new(file.as_str()).into_bytes();
        for path in &self.paths {
            if *path.0 == request.path {
                response = path.1;
            }
        }
        //println!("{:#?}", String::from_utf8_lossy(response));
        match stream.write_all(response) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Failed to write bytes to the stream: {}", e);
            }
        };
    }

    /**
     * # Start listening on a address
     * ```
     * use hddp::Server;
     *
     * fn main() -> std::io::Result<()> {
     *     let server = Server::new();
     *     server.listen("127.0.0.1:8080")?;
     *     Ok(())
     * }
     * ```
     */
    pub fn listen<T: ToSocketAddrs>(&self, addr: T) -> Result<()> {
        let listener = TcpListener::bind(addr)?;

        // accept connections and process them serially
        for stream in listener.incoming() {
            self.handle_client(stream?);
        }
        Ok(())
    }

    /**
     * # Add path to respond
     * ```
     * let mut server = Server::new();
     * server.add_path("/pizza");
     * ```
     */
    pub fn add_path(&mut self, path: &'a str, resp: HttpResponse<'a>) {
        if self.paths.contains_key(&path) {
            self.remove_path(path);
        }
        self.paths.insert(path, resp.into_bytes());
    }

    /**
     * # Remove path to respond
     * ```
     * let mut server = Server::new();
     * server.remove_path("/pizza");
     * ```
     * Note that you can remove "/" if you doesn't want it to return something
     * ```
     * let mut server = Server::new();
     * server.remove_path("/");
     * ```
     */
    pub fn remove_path(&mut self, path: &'a str) {
        if self.paths.contains_key(&path) {
            self.paths.remove(path);
        }
    }

    /**
     * # Create a new server
     * ```
     * let server = Server::new();
     * ```
     */
    pub fn new() -> Self {
        Server::default()
    }
}
