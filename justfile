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

# Start container and open an interactive bash shell in the integrated terminal
shell:
    docker compose up -d dev
    docker compose exec dev bash

# Watch code changes and stream server logs live inside the Linux container
docker-watch:
    docker compose up -d dev
    docker compose exec dev cargo watch -c -w src -d 0.1 -x run

# Run the project inside the Linux container
docker-run:
    docker compose up -d dev
    docker compose exec dev cargo run

# Run tests inside the Linux container
docker-test:
    docker compose exec dev cargo test

# Check code inside the Linux container
docker-check:
    docker compose exec dev cargo check

# Run clippy inside the Linux container
docker-clippy:
    docker compose exec dev cargo clippy --all-targets --all-features

# Follow live container logs in the terminal
docker-logs:
    docker compose logs -f --tail=100 dev

# Start Docker environment in the background
docker-up:
    docker compose up -d

# Stop Docker environment
docker-down:
    docker compose down

# Rebuild Docker container
docker-build:
    docker compose build

# Kill any hanging server process holding port 3000 inside the container
docker-kill:
    docker compose exec dev pkill -9 -f target/debug/http-server || true

# ================================
# Docker Short Aliases
# ================================
d-shell: shell
d-dev: docker-watch
d-watch: docker-watch
d-run: docker-run
d-test: docker-test
d-check: docker-check
d-clippy: docker-clippy
d-logs: docker-logs
d-up: docker-up
d-down: docker-down
d-build: docker-build
d-kill: docker-kill
