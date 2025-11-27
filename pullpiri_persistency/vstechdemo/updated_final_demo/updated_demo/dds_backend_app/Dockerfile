# Multi-stage build for Rust DDS Backend Receiver
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
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

# Set working directory
WORKDIR /app

# Copy the compiled binary from builder stage
COPY --from=builder /app/target/release/dds_backend_receiver /app/dds_backend_receiver

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to app user
USER appuser

# Expose port for DDS backend REST API
EXPOSE 8081

# Set environment variables
ENV RUST_LOG=info

# Run the DDS backend receiver
CMD ["./dds_backend_receiver"]