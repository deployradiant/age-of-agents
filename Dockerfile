# ── Stage 1: Build ──────────────────────────────────────────────────
FROM rust:latest AS builder

WORKDIR /app

# Copy manifest files first for layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies (cached layer)
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true

# Now copy the real source and rebuild (only changed files)
COPY src/ src/
RUN cargo build --release --bin age-of-agents

# ── Stage 2: Runtime ────────────────────────────────────────────────
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies (SSL certs + Python for Modal compatibility)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    python3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/age-of-agents /app/age-of-agents

# Copy the frontend (HTML/JS/CSS assets)
COPY frontend/ /app/frontend/

EXPOSE 8000

ENTRYPOINT ["/app/age-of-agents"]