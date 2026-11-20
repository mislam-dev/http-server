FROM rust:bookworm

# Install Linux system programming, networking, debugging, and testing utilities
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
    netcat-openbsd \
    socat \
    apache2-utils \
    tcpdump \
    htop \
    tree \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install rustfmt and clippy for code formatting and linting
RUN rustup component add rustfmt clippy

# Install just CLI command runner
RUN curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /usr/local/bin --tag 1.38.0

# Install cargo-watch for live reload & watching code changes
RUN ARCH=$(uname -m) && \
    curl -sSL -o /tmp/cargo-watch.deb "https://github.com/watchexec/cargo-watch/releases/download/v8.5.3/cargo-watch-v8.5.3-${ARCH}-unknown-linux-gnu.deb" && \
    dpkg -i /tmp/cargo-watch.deb && \
    rm /tmp/cargo-watch.deb

WORKDIR /app

EXPOSE 3000

# Run server with live reload watching src directory
CMD ["cargo", "watch", "-c", "-w", "src", "-d", "0.1", "-x", "run"]
