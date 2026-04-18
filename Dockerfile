# Build stage
FROM rust:1.94-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Build the real binary
COPY src/ src/
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/proxy-exporter /bin/proxy-exporter

ENV LISTEN_ADDRESS=0.0.0.0:9898
EXPOSE 9898

ENTRYPOINT ["/bin/proxy-exporter"]
