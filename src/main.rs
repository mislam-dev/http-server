use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::{fs, thread};

fn handle_stream(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let mut headers: [httparse::Header<'_>; 64] = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    let _ = req.parse(&buffer).unwrap();

    let method = req.method.unwrap_or("GET");
    let path = req.path.unwrap_or("/");

    println!("{}:{}", method, path);

    let contents = fs::read_to_string("file.html").unwrap_or_default();
    send_response(&mut stream, "200 OK", &contents);
}

fn send_response(stream: &mut TcpStream, status: &str, contents: &str) {
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
        status,
        contents.len(),
        contents
    );

    if let Err(err) = stream.write_all(response.as_bytes()) {
        println!("Failed to send response: {}", err);
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000").expect("failed to run on port 8000");
    println!("Listening on 127.0.0.1:8000");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                thread::spawn(move || handle_stream(s));
            }
            Err(e) => {
                eprintln!("Error expecting connection: {}", e);
            }
        }
    }
}
