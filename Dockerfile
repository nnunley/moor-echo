# Multi-stage Dockerfile for moor-echo

# Build stage
FROM rust:1.75-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
COPY crates/echo-core/Cargo.toml ./crates/echo-core/
COPY crates/echo-repl/Cargo.toml ./crates/echo-repl/
COPY crates/echo-web/Cargo.toml ./crates/echo-web/

# Create dummy source files to cache dependencies
RUN mkdir -p crates/echo-core/src && \
    echo "fn main() {}" > crates/echo-core/src/lib.rs && \
    mkdir -p crates/echo-repl/src && \
    echo "fn main() {}" > crates/echo-repl/src/main.rs && \
    mkdir -p crates/echo-web/src && \
    echo "fn main() {}" > crates/echo-web/src/main.rs

# Build dependencies
RUN cargo build --release

# Copy Node.js files
COPY package.json package-lock.json ./

# Install Node.js dependencies
RUN npm ci

# Copy grammar files for tree-sitter
COPY grammar.js ./
COPY src/grammar.json ./src/
COPY src/node-types.json ./src/
COPY src/parser.c ./src/

# Generate tree-sitter parser
RUN npx tree-sitter generate

# Remove dummy source files
RUN rm -rf crates/*/src

# Copy actual source code
COPY . .

# Build the application
RUN cargo build --release --workspace

# Runtime stage for echo-repl
FROM debian:bookworm-slim AS echo-repl

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 echo

# Copy binary from builder
COPY --from=builder /app/target/release/echo-repl /usr/local/bin/echo-repl

# Set user
USER echo

# Set entrypoint
ENTRYPOINT ["echo-repl"]

# Runtime stage for echo-web
FROM debian:bookworm-slim AS echo-web

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 echo

# Copy binary from builder
COPY --from=builder /app/target/release/echo-web /usr/local/bin/echo-web

# Copy static files
COPY --from=builder /app/crates/echo-web/static /app/static

# Set working directory
WORKDIR /app

# Set user
USER echo

# Expose port
EXPOSE 3000

# Environment variables
ENV HOST=0.0.0.0
ENV PORT=3000
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Set entrypoint
ENTRYPOINT ["echo-web"]

# Development stage
FROM rust:1.75 AS development

# Install development tools
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    nodejs \
    npm \
    git \
    vim \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Rust tools
RUN rustup component add rustfmt clippy rust-analyzer

# Install cargo tools
RUN cargo install cargo-watch cargo-tarpaulin

# Create app directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
COPY package.json package-lock.json ./

# Install dependencies
RUN cargo fetch && npm ci

# Copy source code
COPY . .

# Expose ports
EXPOSE 3000 8080

# Default command
CMD ["bash"]