# Dockerfile
FROM rust:1.82-slim-bookworm

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests first for caching
COPY Cargo.toml Cargo.lock* ./

# Create dummy src for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true
RUN rm -rf src

# Copy source
COPY . .

# Build
RUN cargo build --release

# Install veto to /usr/local/bin for sandbox use
RUN cp target/release/veto /usr/local/bin/veto

# Test entrypoint
CMD ["cargo", "test"]
