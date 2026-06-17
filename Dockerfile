# Build stage
# Base images are pinned by digest (OpenSSF Scorecard: Pinned-Dependencies).
# The :tag is kept alongside the digest for human readability; Dependabot's
# docker ecosystem keeps the digest fresh. Refresh with:
#   docker buildx imagetools inspect <image> --format '{{.Manifest.Digest}}'
FROM rust:1.96-slim@sha256:3b05f7c617a200c41c3506097f0d15fc193a1c93bfd8f141007b47cac8f95d3c AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir -p crates && \
    echo "fn main() {}" > crates/main.rs && \
    echo "pub fn add(a: i64, b: i64) -> i64 { a + b }" > crates/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && \
    rm -rf crates/

# Copy actual source code
COPY crates/ ./crates/

# Build actual binary
RUN cargo build --release

# Runtime stage - use distroless for minimal attack surface.
# Pinned by digest (no :latest) to satisfy Scorecard Pinned-Dependencies and
# Trivy DS-0001; Dependabot's docker ecosystem keeps the digest fresh.
FROM gcr.io/distroless/cc-debian12@sha256:d703b626ba455c4e6c6fbe5f36e6f427c85d51445598d564652a2f334179f96e

# Copy binary from builder
COPY --from=builder /app/target/release/rust_template /usr/local/bin/rust_template

# Set non-root user
USER nonroot:nonroot

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/rust_template"]

# Run the binary
ENTRYPOINT ["/usr/local/bin/rust_template"]
