# Raw HTTP/1.1 Server with Linux `epoll` in Rust

A lightweight, high-performance, event-driven HTTP/1.1 server built entirely from scratch in Rust using low-level POSIX sockets and Linux `epoll`. 

This project intentionally avoids high-level asynchronous runtimes (like Tokio, async-std, or Mio) to demonstrate how non-blocking I/O, OS kernel event notification, and socket buffers work at the systems level.

---

## 🌟 Key Features

- **Pure Systems-Level Architecture:** Built with raw POSIX system calls via the `nix` crate (`socket`, `bind`, `listen`, `accept`, `recv`, `write`, `fcntl`, `epoll`).
- **Zero Async Runtimes:** No `async`/`await`, no hidden thread pools, and no external runtime abstractions.
- **True Non-Blocking I/O:** Both listening and client sockets are configured with `O_NONBLOCK` via `fcntl`.
- **Event-Driven Multiplexing (`epoll`):** Monitors hundreds of concurrent connections using an `epoll_wait` event loop.
- **Connection Backlog Draining:** Calls `accept()` in a loop upon listener readiness until `EAGAIN` to prevent connection starvation.
- **Chunked Read Accumulation:** Drains incoming bytes from the kernel receive buffer until `EAGAIN` to handle fragmented TCP packets.
- **Non-Blocking Writes with `EPOLLOUT`:** Tracks partial writes and pending outbound byte offsets, promoting sockets to `EPOLLOUT` when the kernel send buffer is full instead of burning CPU with busy-wait loops.
- **Cross-Platform Containerization:** Includes a multi-stage Docker environment and `justfile` for seamless development and testing on non-Linux hosts (e.g., macOS).

---

## 🏗️ Architecture & Event Flow

The server runs on a single thread utilizing an event loop driven by `epoll_wait`:

```
                    ┌────────────────────────┐
                    │      epoll_wait()      │ ◄─────────────────────┐
                    └───────────┬────────────┘                       │
                                │                                    │
                  ┌─────────────┴─────────────┐                      │
                  ▼                           ▼                      │
        [Server Socket Event]        [Client Socket Event]           │
                  │                           │                      │
        ┌─────────┴─────────┐       ┌─────────┴─────────┐            │
        │ accept() in loop  │       │  Check Event Type │            │
        │   until EAGAIN    │       └────┬─────────┬────┘            │
        └─────────┬─────────┘            │         │                 │
                  │                      ▼         ▼                 │
     ┌────────────┴────────────┐     [EPOLLIN]  [EPOLLOUT]           │
     │ Set client O_NONBLOCK   │         │         │                 │
     │ Register client EPOLLIN │         │         └───────────────┐ │
     └────────────┬────────────┘         ▼                         ▼ │
                  │               ┌──────────────┐         ┌───────────────┐
                  │               │ recv() loop  │         │ write() loop  │
                  │               │ until EAGAIN │         │ from offset   │
                  │               └──────┬───────┘         └───────┬───────┘
                  │                      ▼                         ▼
                  │               ┌──────────────┐         ┌───────────────┐
                  │               │ Parse HTTP & │         │ Complete?     │
                  │               │ Queue buffer │         │ Yes: Close    │
                  │               └──────┬───────┘         │ No: EPOLLOUT  │
                  │                      │                 └───────┬───────┘
                  └──────────────────────┴─────────────────────────┘
```

### 1. Socket Setup & Non-Blocking Mode
The server socket is initialized with `AF_INET` and `SOCK_STREAM`. Sockets are set to non-blocking via `fcntl`:
```rust
let flags = fcntl(fd, FcntlArg::F_GETFL)?;
fcntl(fd, FcntlArg::F_SETFL(flags | OFlag::O_NONBLOCK))?;
```

### 2. Draining Incoming Connections
When a client connects, `epoll_wait` fires an `EPOLLIN` event on the listener socket. To prevent connection starvation under high concurrent load, `accept()` runs in a loop until it returns `EAGAIN`.

### 3. Reading Request Payloads
Client sockets receive data non-blockingly into a reusable chunk buffer, appending to a connection read buffer until `recv()` yields `EAGAIN` or EOF (`0` bytes).

### 4. Non-Blocking Outbound Writes & `EPOLLOUT`
When responding:
1. The server attempts an immediate non-blocking `write()`.
2. If all bytes are written, the socket is cleanly closed (`Connection: close`).
3. If a partial write occurs or `write()` returns `EAGAIN` (kernel send buffer full), the server updates `pending_writes` with the remaining offset and registers the socket for `EPOLLOUT`.
4. When the kernel buffer clears, `epoll_wait` notifies the server to resume flushing without blocking the event loop.

---

## 📁 Project Structure

```
.
├── Cargo.toml          # Rust dependencies (nix crate for POSIX bindings)
├── Dockerfile          # Linux environment for building & running epoll
├── docker-compose.yml  # Container setup with caching & port mapping (3000:3000)
├── file.html           # Sample static HTML file for GET responses
├── justfile            # Task runner commands (native & Docker aliases)
├── test.http           # REST Client test file
└── src
    ├── httpparse.rs    # HTTP/1.1 request parser (method, path, headers)
    ├── lib.rs          # Library module exports
    ├── main.rs         # Entry point (initializes & starts TcpServer)
    └── tcp_server.rs   # Core non-blocking TCP server & epoll state engine
```

---

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) (2024 edition or newer)
- [Docker](https://www.docker.com/) & Docker Compose (required if running on macOS or Windows, since `epoll` is Linux-specific)
- [Just](https://github.com/casey/just) command runner (optional, but recommended)

---

### Running via Docker (Recommended for macOS / Windows)

Because `epoll` is a Linux kernel subsystem, development on macOS/Windows runs inside the Linux container:

```bash
# Start container and run in watch mode (auto-reloads on file changes)
just d-watch

# Or run directly inside Docker
just d-run

# Open an interactive shell inside the container
just d-shell
```

Alternatively, using standard Docker Compose:
```bash
docker compose up -d app
docker compose exec app cargo run
```

---

### Running Natively (Linux only)

If you are running natively on a Linux machine:

```bash
# Run the server
cargo run

# Or watch for changes
cargo watch -c -w src -d 0.1 -x run
```

The server binds to `0.0.0.0:3000`.

---

## 🧪 Testing the Server

You can test the server using `curl`, modern browsers, or tools like `hey` / `wrk`.

### Basic HTTP Requests

```bash
# GET Request
curl -i http://localhost:3000/

# POST Request
curl -i -X POST http://localhost:3000/ -d '{"hello":"world"}'

# 404 Route
curl -i http://localhost:3000/unknown
```

Expected Response:
```http
HTTP/1.1 200 OK
Content-Type: text/html
Content-Length: 178
Connection: close

<!DOCTYPE html>
<html>
...
```

---

## ⚡ Benchmarking Concurrency

To benchmark throughput and concurrent connection handling using [`wrk`](https://github.com/wg/wrk) or [`hey`](https://github.com/rakyll/hey):

```bash
# 10,000 requests across 100 concurrent connections
wrk -t4 -c100 -d10s http://localhost:3000/
```

---

## 📚 Technical Concepts Explored

- **POSIX Sockets & File Descriptors:** Low-level network endpoints, socket options (`SO_REUSEADDR`, `SO_REUSEPORT`), and descriptor lifetimes.
- **Kernel I/O Buffers (`SO_RCVBUF`, `SO_SNDBUF`):** How packet flow control and window sizes cause `EAGAIN` / `EWOULDBLOCK`.
- **Level-Triggered vs. Edge-Triggered Epoll:** Managing event notifications, avoiding busy-polling, and draining buffers cleanly.
- **The C10K Problem:** Why single-threaded non-blocking event loops drastically outperform thread-per-connection architectures under heavy concurrency.
- **Rust I/O Safety (`AsFd`, `BorrowedFd`, `OwnedFd`):** Safely managing raw file descriptors in modern Rust.

---

## 📄 License

MIT License. Feel free to use and modify for learning and systems programming experiments.
