# Default: list all available commands
default:
    @just --list

# ================================
# Local Development (Host)
# ================================

# Run the project locally
start:
    cargo run

# Watch code changes and run locally
dev:
    cargo watch -c -w src -d 0.1 -x run

# Run tests locally
test:
    cargo test

# Check code locally
check:
    cargo check

# Run clippy with warnings treated as errors
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Format code
fmt:
    cargo fmt

# Lint code without failing on warnings
lint:
    cargo clippy --all-targets --all-features

# Run complete CI checks locally
ci:
    cargo fmt --check
    cargo check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test

# ================================
# Docker / Linux Environment
# ================================

# Start server in Linux container with hot reload & stream live logs
docker-watch:
    docker compose up

# Start Linux container in background
docker-up:
    docker compose up -d

# Follow live container logs in the terminal
docker-logs:
    docker compose logs -f

# Stop Linux container
docker-down:
    docker compose down

# Run the project once inside the Linux container
docker-run:
    docker compose run --rm --service-ports app cargo run

# Run tests inside the Linux container
docker-test:
    docker compose run --rm app cargo test

# Check code inside the Linux container
docker-check:
    docker compose run --rm app cargo check

# Run clippy inside the Linux container
docker-clippy:
    docker compose run --rm app cargo clippy --all-targets --all-features

# Rebuild Docker image
docker-build:
    docker compose build

# ================================
# Docker Short Aliases
# ================================
up: docker-watch
up-d: docker-up
logs: docker-logs
down: docker-down
build: docker-build

d-watch: docker-watch
d-dev: docker-watch
d-up: docker-up
d-down: docker-down
d-logs: docker-logs
d-run: docker-run
d-test: docker-test
d-check: docker-check
d-clippy: docker-clippy
d-build: docker-build
