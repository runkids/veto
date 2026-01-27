# Dockerfile
FROM rust:1.82-slim-bookworm

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    ca-certificates \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js 22 (required for molt.bot)
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install molt.bot (moltbot CLI) via npm
RUN npm install -g clawdbot

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
