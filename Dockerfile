# ==============================================================================
# Multi-stage Dockerfile for LC-3 Assembler
# ==============================================================================
# Produces a minimal Docker image (~10MB) for running the LC-3 assembler
#
# Usage:
#   docker build -t lc3-assembler .
#   docker run -v $(pwd):/workspace lc3-assembler program.asm
# ==============================================================================

# ==============================================================================
# Stage 1: Builder
# ==============================================================================
FROM rust:1.75 as builder

# Set working directory
WORKDIR /build

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Copy source code
COPY src ./src
COPY tests ./tests

# Build release binary
RUN cargo build --release

# Strip debug symbols to reduce size
RUN strip /build/target/release/lc3-assembler

# ==============================================================================
# Stage 2: Runtime
# ==============================================================================
FROM debian:bookworm-slim

# Install minimal dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/target/release/lc3-assembler /usr/local/bin/lc3-assembler

# Create workspace directory
WORKDIR /workspace

# Set up entry point
ENTRYPOINT ["lc3-assembler"]

# Default command (show help)
CMD ["--help"]

# Metadata
LABEL maintainer="LC-3 Assembler Contributors"
LABEL description="LC-3 (Little Computer 3) Assembler"
LABEL version="1.0.0"
