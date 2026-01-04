pub mod request;

use request::{HttpRequest, HttpResponse};

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::io::{Read, Result, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::{fs, thread};
use std::sync::{Arc, Mutex};

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
pub struct Server {
    paths: Arc<Mutex<HashMap<String, HashMap<String, Vec<u8>>>>>,
    not_found: Arc<Vec<u8>>,  
}

impl Default for Server {
    fn default() -> Self {
        let mut paths = HashMap::new();
        let mut methods = HashMap::new();
        let file = match fs::read_to_string("pages/default/index.html") {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Failed to read the file: {}", e);
                "default".to_string()
            }
        };

        let file_not_found = match fs::read_to_string("pages/404/index.html") {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Failed to read the file: {}", e);
                "404".to_string()
            }
        };
        let response = HttpResponse::new(file.as_str());
        let mut response_not_found = HttpResponse::new(file_not_found.as_str());
        response_not_found.change_status_line("HTTP/1.1 404 NOT FOUND");
        methods.insert("GET".to_string(), response.into_bytes());
        paths.insert("/".to_string(), methods);
        Server { paths: Arc::new(Mutex::new(paths)), not_found: Arc::new(response_not_found.into_bytes()) }
    }
}

impl Server {
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
                eprintln!("Failed to serialized as an HttpRequest: {}", e);
                return;
            }
        };
        println!("{} {}", request.method, request.path);
        
        let map = self.paths.lock().unwrap();
        
        let response = map.get(request.path)
        .and_then(|methods| methods.get(request.method))
        .unwrap_or(&self.not_found);
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
    pub fn listen<T: ToSocketAddrs + Display>(&self, addr: T) -> Result<()> {
        let listener = TcpListener::bind(&addr)?;

        println!("Started Listening on http://{}", addr);
        // accept connections and process them serially
        for stream_result in listener.incoming() {
            let stream = stream_result.expect("");
            let server_clone = self.clone();
            thread::spawn(move || {
                server_clone.handle_client(stream);
            });
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
    pub fn add_path(&mut self, method: &str, path: &str, resp: HttpResponse) {
        let path = path.to_string();
        let method = method.to_string();
        let mut method_map = HashMap::new();
        let mut map = self.paths.lock().unwrap();
        method_map.insert(method.clone(), resp.into_bytes());
        map
            .entry(path)
            .or_insert(method_map)
            .insert(method, resp.into_bytes());
    }

    pub fn remove_method(&mut self, method: &str, path: &str) {
        let path = path.to_string();
        let mut map = self.paths.lock().unwrap();
        if let Some(method_map) = map.get_mut(&path) {
            method_map.remove(method);
        }
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
    pub fn remove_path(&mut self, path: &str) {
        let path = path.to_string();
        let mut map = self.paths.lock().unwrap();
        map.remove(&path);
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
