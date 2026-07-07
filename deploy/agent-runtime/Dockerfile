# agent-server with optional PostgreSQL (`AGENT_RUNTIME_DATABASE_URL`).
FROM rust:1.88-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY deploy/agent-runtime/schema.sql deploy/agent-runtime/schema.sql
RUN cargo build --release -p agent-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/agent-server /usr/local/bin/agent-server
ENV AGENT_RUNTIME_PORT=8787
EXPOSE 8787
CMD ["agent-server"]
