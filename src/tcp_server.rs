use super::httpparse::{HttpParse, Request};
use nix::libc::close;
use nix::sys::socket::sockopt::{ReuseAddr, ReusePort};
use nix::sys::socket::{
    AddressFamily, Backlog, MsgFlags, SockFlag, SockProtocol, SockType, SockaddrIn, accept, bind,
    listen, recv, send, setsockopt, socket,
};
use std::os::fd::OwnedFd;
use std::os::fd::{AsRawFd, RawFd};

pub struct TcpServer {
    port: u16,
}

impl TcpServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn start(&self) {
        let server_socket = self.create_socket();
        self.bind(server_socket.as_raw_fd());
        self.listen(&server_socket);
        self.process(server_socket.as_raw_fd());
    }

    fn listen(&self, server_socket: &OwnedFd) {
        listen(&server_socket, Backlog::new(5).unwrap()).expect("failed to listen on the socket");
        println!("server is running on port :{}", self.port);
    }

    fn bind(&self, fd: RawFd) {
        let addr = SockaddrIn::new(127, 0, 0, 1, self.port);
        bind(fd, &addr).expect("failed to bind with ip address");
    }

    fn create_socket(&self) -> OwnedFd {
        let flags = SockFlag::empty();
        let server_socket = socket(
            AddressFamily::Inet,
            SockType::Stream,
            flags,
            SockProtocol::Tcp,
        )
        .expect("Failed to start socket");

        setsockopt(&server_socket, ReuseAddr, &true).unwrap();
        setsockopt(&server_socket, ReusePort, &true).unwrap();

        if server_socket.as_raw_fd() == -1 {
            println!("invalid socket");
            std::process::exit(1);
        }
        return server_socket;
    }

    fn accept_and_receive_request(
        &self,
        server_fd: RawFd,
    ) -> Result<(i32, [u8; 65536], usize), String> {
        let client_fd = match accept(server_fd) {
            Ok(fd) => fd,
            Err(e) => {
                return Err(format!("failed to accept connection: {}", e));
            }
        };
        if client_fd == -1 {
            eprintln!("failed to accept connection: invalid client_fd");
        }

        let mut buffer = [0u8; 65536];
        let bytes_receive = match recv(client_fd, &mut buffer, MsgFlags::empty()) {
            Ok(r) => r,
            Err(e) => {
                return Err(format!("failed to receive data: {}", e));
            }
        };
        Ok((client_fd, buffer, bytes_receive))
    }

    fn handle_request(&self, req: Request) -> Result<String, String> {
        let method = req.method;

        let mut status = "200 OK";
        let mut content_type = "text/html";
        let mut content = String::new();

        if method == "GET" {
            content = std::fs::read_to_string("file.html").unwrap();
        } else if method == "POST" {
            status = "201 Created";
            content_type = "applications/json";
            content = String::from(
                r#"{"name":"JohnSmith","age":27,"isAlive":true,"address":{"streetAddress":"212ndStreet","city":"NewYork","state":"NY","postalCode":"10021-3100"},"phoneNumbers":[{"type":"home","number":"212555-1234"},{"type":"office","number":"646555-4567"}],"children":["Catherine","Thomas","Trevor"],"spouse":null}"#,
            );
        } else {
            status = "404 Not Found";
            content_type = "applications/json";
        }

        let response = format!(
            "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            status,
            content_type,
            content.len(),
            content
        );
        Ok(response)
    }

    fn process(&self, fd: RawFd) {
        loop {
            let _ = unsafe {
                let accept_and_rec = self.accept_and_receive_request(fd);

                if let Err(_) = accept_and_rec {
                    continue;
                }
                let (client_fd, buffer, bytes_receive) = accept_and_rec.unwrap();

                let received_text = std::str::from_utf8(&buffer[..bytes_receive]).unwrap();

                let req = HttpParse::parse(received_text);

                match self.handle_request(req) {
                    Ok(response) => {
                        let _bytes_send = send(client_fd, response.as_bytes(), MsgFlags::empty())
                            .expect("failed to send message");
                    }
                    Err(e) => {
                        println!("failed to process request: {}", e);
                        continue;
                    }
                }

                close(client_fd);
            };
        }
    }
}
