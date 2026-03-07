# Development container for veto (Rust CLI)
# Multi-stage: pre-download deps for faster rebuilds

FROM rust:1.87-bookworm

# System tools
RUN apt-get update && apt-get install -y \
    bash \
    git \
    curl \
    jq \
    sudo \
    procps \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install useful Rust dev tools
RUN cargo install cargo-watch cargo-nextest

# Create non-root developer user
RUN groupadd -g 1000 developer \
    && useradd -m -u 1000 -g developer -s /bin/bash developer \
    && echo "developer ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers.d/developer

WORKDIR /workspace

# Pre-download dependencies for caching
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build 2>/dev/null || true \
    && cargo build --release 2>/dev/null || true \
    && rm -rf src

# Fix ownership
RUN chown -R developer:developer /workspace

USER developer

CMD ["sleep", "infinity"]
