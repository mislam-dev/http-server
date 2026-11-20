use super::httpparse::{HttpParse, Request};
use nix::errno::Errno::EAGAIN;
use nix::fcntl::{FcntlArg, OFlag, fcntl};
use nix::libc::close;
use nix::sys::epoll::{Epoll, EpollCreateFlags, EpollEvent, EpollFlags, EpollTimeout};
use nix::sys::socket::sockopt::{ReuseAddr, ReusePort};
use nix::sys::socket::{
    AddressFamily, Backlog, MsgFlags, SockFlag, SockProtocol, SockType, SockaddrIn, accept, bind,
    listen, recv, setsockopt, socket,
};
use nix::unistd::write;
use std::collections::HashMap;
use std::os::fd::{AsRawFd, RawFd};
use std::os::fd::{BorrowedFd, OwnedFd};

pub struct TcpServer {
    port: u16,
}

impl TcpServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn start(&self) {
        let server_socket = self.create_socket();
        self.set_non_blocking(server_socket.as_raw_fd());
        self.bind(server_socket.as_raw_fd());
        self.listen(&server_socket);

        let epoll = Epoll::new(EpollCreateFlags::empty()).expect("failed to create epoll instance");

        let server_fd = server_socket.as_raw_fd();
        let server_event = EpollEvent::new(EpollFlags::EPOLLIN, server_fd as u64);

        epoll
            .add(&server_socket, server_event)
            .expect("failed to register server event on epoll");

        self.process(&epoll, server_socket.as_raw_fd());
    }

    fn set_non_blocking(&self, fd: RawFd) {
        let borrowed_server_fd = unsafe { BorrowedFd::borrow_raw(fd) };
        let flags =
            fcntl(borrowed_server_fd, FcntlArg::F_GETFL).expect("failed to get fcntl flags");
        let new_flags = OFlag::from_bits_truncate(flags) | OFlag::O_NONBLOCK;
        let _ = fcntl(borrowed_server_fd, FcntlArg::F_SETFL(new_flags))
            .expect("failed to set non_blocking");
    }

    fn listen(&self, server_socket: &OwnedFd) {
        listen(&server_socket, Backlog::new(5).unwrap()).expect("failed to listen on the socket");
        println!("server is running on port :{}", self.port);
    }

    fn bind(&self, fd: RawFd) {
        let addr = SockaddrIn::new(0, 0, 0, 0, self.port);
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
        server_socket
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
            "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status,
            content_type,
            content.len(),
            content
        );
        Ok(response)
    }

    fn handle_accept(&self, epoll: &Epoll, server_fd: RawFd) {
        loop {
            match accept(server_fd) {
                Ok(client_fd) => {
                    self.set_non_blocking(client_fd);
                    let client_borrowed = unsafe { BorrowedFd::borrow_raw(client_fd) };
                    let event = EpollEvent::new(
                        EpollFlags::EPOLLIN | EpollFlags::EPOLLRDHUP,
                        client_fd as u64,
                    );
                    if let Err(e) = epoll.add(client_borrowed, event) {
                        eprintln!("failed to register new incoming request: {e}");
                        unsafe {
                            close(client_fd);
                        }
                    }
                }
                Err(EAGAIN) => {
                    break;
                }
                Err(e) => {
                    eprintln!("failed to accept new request: {}", e);
                    break;
                }
            }
        }
    }

    fn handle_client(
        &self,
        epoll: &Epoll,
        client_fd: RawFd,
        events: EpollFlags,
        pending_writes: &mut HashMap<RawFd, (Vec<u8>, usize)>,
    ) {
        let client_fd_borrowed = unsafe { BorrowedFd::borrow_raw(client_fd) };

        if events.intersects(EpollFlags::EPOLLRDHUP | EpollFlags::EPOLLHUP | EpollFlags::EPOLLERR) {
            let _ = epoll.delete(client_fd_borrowed);
            unsafe {
                close(client_fd);
            }
            return;
        }
        if events.contains(EpollFlags::EPOLLIN) {
            let mut connection_closed: bool = false;
            let mut read_buffer = vec![];
            let mut chunk = [0u8; 65536];

            loop {
                match recv(client_fd, &mut chunk, MsgFlags::empty()) {
                    Ok(0) => {
                        connection_closed = true;
                        break;
                    }
                    Ok(n) => {
                        // process the request
                        read_buffer.extend_from_slice(&chunk[..n]);
                    }
                    Err(EAGAIN) => {
                        break;
                    }

                    Err(e) => {
                        eprintln!("failed to revieved data: {e}");
                        connection_closed = true;
                        break;
                    }
                }
            }

            if connection_closed {
                let _ = epoll.delete(client_fd_borrowed);
                unsafe {
                    close(client_fd);
                }
                return;
            }

            // send to client non blocking way
            let received_text = std::str::from_utf8(&read_buffer).unwrap();
            let req = HttpParse::parse(received_text);

            match self.handle_request(req) {
                Ok(response) => {
                    pending_writes.insert(client_fd, (response.as_bytes().to_vec(), 0));

                    self.handle_write(epoll, client_fd, pending_writes);
                }

                Err(e) => {
                    println!("failed to process request: {}", e);
                }
            }
            return;
        }

        if events.contains(EpollFlags::EPOLLOUT) {
            self.handle_write(epoll, client_fd, pending_writes);
        }
    }

    fn handle_write(
        &self,
        epoll: &Epoll,
        client_fd: RawFd,
        pending_writes: &mut HashMap<RawFd, (Vec<u8>, usize)>,
    ) {
        let client_fd_borrowed = unsafe { BorrowedFd::borrow_raw(client_fd) };

        if let Some((bytes, total_sent)) = pending_writes.get_mut(&client_fd) {
            while *total_sent < bytes.len() {
                match write(client_fd_borrowed, &bytes[*total_sent..]) {
                    Ok(n) => {
                        *total_sent += n;
                    }
                    Err(EAGAIN) => {
                        let mut event = EpollEvent::new(
                            EpollFlags::EPOLLOUT | EpollFlags::EPOLLRDHUP,
                            client_fd as u64,
                        );
                        let _ = epoll.modify(client_fd_borrowed, &mut event);
                        return;
                    }
                    Err(e) => {
                        eprintln!("failed to write data: {e}");
                        pending_writes.remove(&client_fd);
                        let _ = epoll.delete(client_fd_borrowed);
                        unsafe {
                            close(client_fd);
                        }
                        return;
                    }
                }
            }
        }
        pending_writes.remove(&client_fd);
        let _ = epoll.delete(client_fd_borrowed);
        unsafe {
            close(client_fd);
        }
    }

    fn process(&self, epoll: &Epoll, server_fd: RawFd) {
        let mut events = [EpollEvent::empty(); 1024];
        let mut pending_writes: HashMap<RawFd, (Vec<u8>, usize)> = HashMap::new();
        loop {
            let num_events = match epoll.wait(&mut events, EpollTimeout::NONE) {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("failed get events:{e}");
                    continue;
                }
            };

            for event in &events[..num_events] {
                let event_fd = event.data() as RawFd;

                if event_fd == server_fd {
                    // new request incoming
                    self.handle_accept(epoll, server_fd);
                } else {
                    // event is on existing client
                    self.handle_client(epoll, event_fd, event.events(), &mut pending_writes);
                }
            }
        }
    }
}
