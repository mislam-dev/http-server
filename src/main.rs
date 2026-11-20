use http_server::tcp_server::TcpServer;

#[warn(non_snake_case)]
fn main() -> nix::Result<()> {
    let tcp = TcpServer::new(3000);
    tcp.start();

    Ok(())
}
