FROM rust:1.90-slim AS builder

WORKDIR /app
COPY . .

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        libssl-dev \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/llm-cache-service /app/app
COPY --from=builder /app/prompts /app/prompts
RUN mkdir -p /app/data

CMD ["./app"]
