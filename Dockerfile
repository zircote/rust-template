# Build stage
# Alpine is musl-native, so `cargo build` produces a fully STATIC binary —
# letting the runtime stage be distroless/static (no glibc/openssl/libstdc++)
# and eliminating the base-image OS-package CVEs a glibc image (distroless/cc)
# otherwise carries.
#
# Base images are pinned by digest (OpenSSF Scorecard: Pinned-Dependencies).
# The :tag is kept alongside the digest for human readability; Dependabot's
# docker ecosystem keeps the digest fresh. Refresh with:
#   docker buildx imagetools inspect <image> --format '{{.Manifest.Digest}}'
FROM rust:1.98-alpine@sha256:a10e64dd139b7387337c7fbe8aca31b959b57b2fd4c8ae20a02cf1d6ea424dce AS builder

# musl-dev + build-base provide the C toolchain/assembler for any C-backed
# crates a downstream project adds (e.g. ring); the musl target links them
# statically. (A pure-Rust project needs neither, but the template should
# build out of the box once dependencies are added.)
RUN apk add --no-cache musl-dev build-base

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies. The explicit [[bench]] target
# (benches/benchmarks.rs) must exist or cargo fails at manifest parse, so stub
# it too — not just crates/.
RUN mkdir -p crates benches && \
    echo "fn main() {}" > crates/main.rs && \
    echo "pub fn add(a: i64, b: i64) -> i64 { a + b }" > crates/lib.rs && \
    echo "fn main() {}" > benches/benchmarks.rs

# Build dependencies (this layer will be cached). Keep the benches/ stub: it
# is .dockerignored (not in the build context), but the explicit [[bench]]
# target must exist for the manifest to parse. The release build never
# compiles it.
RUN cargo build --release && \
    rm -rf crates/

# Copy actual source code (benches/ stub stays from the step above)
COPY crates/ ./crates/

# Build actual binary
RUN cargo build --release

# Runtime stage - distroless/static (no glibc/openssl) for the static musl
# binary. Pinned by digest (no :latest) to satisfy Scorecard
# Pinned-Dependencies and Trivy DS-0001; Dependabot keeps the digest fresh.
FROM gcr.io/distroless/static-debian12:nonroot@sha256:b7bb25d9f7c31d2bdd1982feb4dafcaf137703c7075dbe2febb41c24212b946f

# Copy the statically-linked binary from builder
COPY --from=builder /app/target/release/rust_template /usr/local/bin/rust_template

# Set non-root user
USER nonroot:nonroot

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/rust_template"]

# Run the binary
ENTRYPOINT ["/usr/local/bin/rust_template"]
