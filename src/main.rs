use http_server::tcp_server::TcpServer;
use nix::sys::epoll;

fn main() {
    let tcp = TcpServer::new(3000);
    tcp.start();
}
