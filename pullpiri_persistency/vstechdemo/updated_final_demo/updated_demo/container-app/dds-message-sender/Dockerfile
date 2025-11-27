# Multi-stage build for Rust DDS Message Sender
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

# Create app user
RUN useradd -r -s /bin/false appuser

# Set working directory
WORKDIR /app

# Copy the compiled binary from builder stage
COPY --from=builder /app/target/release/dds_message_sender /app/dds_message_sender

# Create data directory for DDS message sharing
RUN mkdir -p /data && chown appuser:appuser /data

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to app user
USER appuser

# Set environment variables
ENV RUST_LOG=info

# Run the DDS message sender
CMD ["./dds_message_sender"]