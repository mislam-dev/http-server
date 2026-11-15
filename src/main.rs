use http_server::httpparse::HttpParse;
use nix::{
    libc::close,
    sys::socket::{
        AddressFamily, Backlog, MsgFlags, SockFlag, SockProtocol, SockType, SockaddrIn, accept,
        bind, listen, recv, send, socket,
    },
};
use std::os::fd::AsRawFd;

fn main() {
    let flags = SockFlag::empty();
    let server_socket = socket(
        AddressFamily::Inet,
        SockType::Stream,
        flags,
        SockProtocol::Tcp,
    )
    .expect("Failed to start socket");

    if server_socket.as_raw_fd() == -1 {
        println!("invalid socket");
        std::process::exit(1);
    }

    let port = 3000;

    let addr = SockaddrIn::new(127, 0, 0, 1, port);

    let _ = bind(server_socket.as_raw_fd(), &addr).expect("failed to bind with ip address");

    listen(&server_socket, Backlog::new(5).unwrap()).expect("failed to listen on the socket");

    println!("server is running on port :{}", port);

    loop {
        let _ = unsafe {
            let client_fd = match accept(server_socket.as_raw_fd()) {
                Ok(fd) => fd,
                Err(e) => {
                    eprintln!("failed to accept connection: {}", e);
                    continue;
                }
            };
            if client_fd == -1 {
                eprintln!("failed to accept connection: invalid client_fd");
                continue;
            }
            println!("accepting connection connection");
            let mut buffer = [0u8; 1024];
            let bytes_receive = match recv(client_fd, &mut buffer, MsgFlags::empty()) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("failed to receive data: {}", e);
                    continue;
                }
            };
            let valid_data = std::str::from_utf8(&buffer[..bytes_receive]).unwrap();
            // println!("receiving bytes: {:#?}", valid_data);
            let headers = HttpParse::parse(valid_data);

            let method = headers.get("method").unwrap_or_else(|| &"GET");

            let mut status = "200 OK";
            let mut content_type = "text/html";
            let mut content = String::new();

            if method == &"GET" {
                content = std::fs::read_to_string("file.html").unwrap();
            } else if method == &"POST" {
                status = "201 Created";
                content_type = "applications/json";
                content = String::from(
                    r#"{"name":"JohnSmith","age":27,"isAlive":true,"address":{"streetAddress":"212ndStreet","city":"NewYork","state":"NY","postalCode":"10021-3100"},"phoneNumbers":[{"type":"home","number":"212555-1234"},{"type":"office","number":"646555-4567"}],"children":["Catherine","Thomas","Trevor"],"spouse":null}"#,
                );
                println!("POST");
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
            let _bytes_send = send(client_fd, response.as_bytes(), MsgFlags::empty())
                .expect("failed to send message");

            close(client_fd);
        };
    }
}
