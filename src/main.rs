use hddp::{Server, request::HttpResponse};
fn main() -> std::io::Result<()> {
    let mut server = Server::new();
    let mut response_post = HttpResponse::new("<h1>POST Reçu !</h1>");
    server.add_path("GET", "/pizza", HttpResponse::default());
    server.add_path("POST", "/test", response_post);
    server.listen("127.0.0.1:8080")?;
    Ok(())
}
