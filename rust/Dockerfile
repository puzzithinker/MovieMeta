# Multi-stage Docker build for Movie Data Capture (Rust)

# Stage 1: Builder
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY mdc-core/Cargo.toml ./mdc-core/
COPY mdc-scraper/Cargo.toml ./mdc-scraper/
COPY mdc-image/Cargo.toml ./mdc-image/
COPY mdc-storage/Cargo.toml ./mdc-storage/
COPY mdc-api/Cargo.toml ./mdc-api/
COPY mdc-cli/Cargo.toml ./mdc-cli/

# Create dummy source files to build dependencies
RUN mkdir -p mdc-core/src mdc-scraper/src mdc-image/src mdc-storage/src mdc-api/src mdc-cli/src && \
    echo "fn main() {}" > mdc-cli/src/main.rs && \
    echo "fn main() {}" > mdc-api/src/bin/mdc-server.rs && \
    echo "pub fn dummy() {}" > mdc-core/src/lib.rs && \
    echo "pub fn dummy() {}" > mdc-scraper/src/lib.rs && \
    echo "pub fn dummy() {}" > mdc-image/src/lib.rs && \
    echo "pub fn dummy() {}" > mdc-storage/src/lib.rs && \
    echo "pub fn dummy() {}" > mdc-api/src/lib.rs

# Build dependencies only
RUN cargo build --release --workspace

# Remove dummy sources
RUN rm -rf mdc-*/src

# Copy actual source code
COPY mdc-core ./mdc-core
COPY mdc-scraper ./mdc-scraper
COPY mdc-image ./mdc-image
COPY mdc-storage ./mdc-storage
COPY mdc-api ./mdc-api
COPY mdc-cli ./mdc-cli

# Build for release
RUN cargo build --release --workspace

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install Python dependencies for CloudScraper bridge
RUN pip3 install --no-cache-dir cloudscraper requests --break-system-packages

# Create app directory
WORKDIR /app

# Create data directories
RUN mkdir -p /app/data /app/output /app/logs

# Copy binaries from builder
COPY --from=builder /app/target/release/mdc-cli /usr/local/bin/
COPY --from=builder /app/target/release/mdc-server /usr/local/bin/

# Copy Python bridge scripts
COPY mdc-scraper/cloudflare_bridge.py /app/
COPY mdc-image/face_detect_bridge.py /app/

# Set environment variables
ENV RUST_LOG=info
ENV MDC_DATA_DIR=/app/data
ENV MDC_OUTPUT_DIR=/app/output
ENV MDC_LOG_DIR=/app/logs

# Expose API port
EXPOSE 3000

# Default command (run server)
CMD ["mdc-server", "--host", "0.0.0.0", "--port", "3000"]
