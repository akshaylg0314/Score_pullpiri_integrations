# Multi-stage build for Rust File Message Writer
FROM rust:1.83 as builder

# Set working directory
WORKDIR /app

# Copy source code
COPY Cargo.toml Cargo.lock* ./
COPY src/ ./src/

# Build the application in release mode
RUN cargo build --release

# Runtime stage with minimal base image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user and data directory
RUN useradd -r -s /bin/false appuser
RUN mkdir -p /data /data/history && chown -R appuser:appuser /data

# Set working directory
WORKDIR /app

# Copy the compiled binary from builder stage
COPY --from=builder /app/target/release/file_message_writer /app/file_message_writer

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to app user
USER appuser

# Run the file message writer
CMD ["./file_message_writer"]