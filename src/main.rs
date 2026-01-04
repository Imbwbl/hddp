use hddp::Server;
fn main() -> std::io::Result<()> {
    let server = Server::new();
    server.listen("127.0.0.1:8080")?;
    Ok(())
}
