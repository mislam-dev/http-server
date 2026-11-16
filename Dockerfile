FROM rust:bookworm

# Install Linux system programming, networking, and debugging tools
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    gdb \
    lldb \
    strace \
    lsof \
    curl \
    net-tools \
    iproute2 \
    procps \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-watch for live reload & watching logs
RUN cargo install cargo-watch --locked

WORKDIR /app

# Keep container running for interactive development
CMD ["sleep", "infinity"]
