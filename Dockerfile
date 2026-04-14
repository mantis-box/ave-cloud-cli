# =============================================================================
# Stage 1: Builder
# =============================================================================
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-tools \
    gcc \
    libc6-dev \
    gcc-aarch64-linux-gnu \
    gcc-arm-linux-gnueabihf \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock* ./

# Create dummy source for dependency caching
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "// lib" > src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release --features rustls 2>/dev/null || true

# Copy actual source
COPY src/ src/
RUN if [ -d .cargo ]; then cp -r .cargo ./.cargo; fi

# Build for multiple targets
# Default: musl static binary (works on most Linux)
RUN cargo build --release --target x86_64-unknown-linux-musl --features rustls && \
    strip target/x86_64-unknown-linux-musl/release/ave-cloud-rs-cli

# =============================================================================
# Stage 2: Runtime
# =============================================================================
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd --create-home --shell /bin/bash ave-cloud-rs-cli
WORKDIR /home/ave-cloud-rs-cli

# Copy binary from builder
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/ave-cloud-rs-cli /usr/local/bin/

# Set permissions
RUN chmod +x /usr/local/bin/ave-cloud-rs-cli && \
    chown ave-cloud-rs-cli:ave-cloud-rs-cli /usr/local/bin/ave-cloud-rs-cli

# Copy service file
COPY scripts/ave-cloud-rs-cli.service /etc/systemd/system/

# Create env file template
RUN mkdir -p /etc/ave-cloud-rs-cli && \
    echo '# AVE_API_KEY=your_api_key_here' > /etc/ave-cloud-rs-cli/env.template && \
    echo '# API_PLAN=free' >> /etc/ave-cloud-rs-cli/env.template && \
    chown -R ave-cloud-rs-cli:ave-cloud-rs-cli /etc/ave-cloud-rs-cli && \
    chmod 600 /etc/ave-cloud-rs-cli/env.template

# Switch to non-root user
USER ave-cloud-rs-cli

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD /usr/local/bin/ave-cloud-rs-cli price BSC 0x0000000000000000000000000000000000000000 || true

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/ave-cloud-rs-cli"]
CMD ["daemon", "--skill", "data-wss"]
