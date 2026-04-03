FROM rust:1.89-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates sqlite3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/are_you_going-cli /app/are_you_going-cli
COPY --from=builder /app/config /app/config
COPY --from=builder /app/assets /app/assets

RUN mkdir -p /app/data

ENV LOCO_ENV=production

CMD ["/app/are_you_going-cli", "start", "--all", "-e", "production"]
