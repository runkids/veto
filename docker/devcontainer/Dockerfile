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
RUN cargo install --locked cargo-watch
RUN curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C /usr/local/cargo/bin

# Create non-root developer user
RUN groupadd -g 1000 developer \
    && useradd -m -u 1000 -g developer -s /bin/bash developer \
    && echo "developer ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers.d/developer

WORKDIR /workspace

# Pre-download dependencies for caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --locked 2>/dev/null || true \
    && cargo build --locked --release 2>/dev/null || true \
    && rm -rf src

# Fix ownership
RUN chown -R developer:developer /workspace

USER developer
RUN mkdir -p ~/.local/bin
ENV PATH="/home/developer/.local/bin:${PATH}"

# Dev shortcuts in bashrc
RUN cat >> ~/.bashrc << 'BASHRC'

# veto dev environment
help() { make -C /workspace help 2>/dev/null || echo "Run from /workspace"; }
alias t='cargo test'
alias tw='cargo watch -x test'
alias r='cargo run --'
alias rr='cargo run --release --'
alias ck='make -C /workspace check'
alias e2e='make -C /workspace e2e'
BASHRC

CMD ["sleep", "infinity"]
