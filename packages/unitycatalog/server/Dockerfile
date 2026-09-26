# syntax=docker/dockerfile:1

FROM rust:1.95-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY cornucopia ./cornucopia
COPY src ./src
COPY queries ./queries
COPY schema.sql cornucopia.toml config.toml ./

RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

RUN useradd --system --uid 10001 --create-home --home-dir /home/unitycatalog unitycatalog
WORKDIR /home/unitycatalog

COPY --from=builder /app/target/release/server /usr/local/bin/unitycatalog-server
COPY --from=builder /app/target/release/uc /home/unitycatalog/bin/uc
COPY --from=builder /app/config.toml /etc/unitycatalog/config.toml

USER unitycatalog
ENV UNITYCATALOG_CONFIG=/etc/unitycatalog/config.toml
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/unitycatalog-server"]
