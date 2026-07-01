# ============================================================
# Stage 1: Planner — cache dependencies
# ============================================================
FROM rust:1-slim-bookworm AS planner
WORKDIR /app
RUN cargo init --name ai_resume_aid
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release 2>/dev/null; true

# ============================================================
# Stage 2: Builder — compile with cached deps
# ============================================================
FROM rust:1-slim-bookworm AS builder
WORKDIR /app
COPY --from=planner /app/target /app/target
COPY --from=planner /usr/local/cargo /usr/local/cargo
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --offline

# ============================================================
# Stage 3: Runtime — minimal image
# ============================================================
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/ai_resume_aid .
COPY --from=builder /app/fonts ./fonts

# Default API prefix for micro_proxy deployment (can be overridden at runtime)
ENV API_PREFIX=/resume-aid-api
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=3001

EXPOSE 3001
CMD ["./ai_resume_aid"]
