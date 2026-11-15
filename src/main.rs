use http_server::tcp_server::TcpServer;

fn main() {
    let tcp = TcpServer::new(3000);
    tcp.start();
}
