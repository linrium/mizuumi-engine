# syntax=docker/dockerfile:1

FROM rust:1.95-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY chronicle-audit ./chronicle-audit
COPY chronicle-audit-macros ./chronicle-audit-macros
COPY cornucopia ./cornucopia
COPY src ./src
COPY queries ./queries
COPY schema.sql cornucopia.toml config.toml ./

RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

RUN useradd --system --uid 10001 --create-home --home-dir /home/controlplane controlplane
WORKDIR /home/controlplane

COPY --from=builder /app/target/release/controlplane-server /usr/local/bin/controlplane-server
COPY --from=builder /app/config.toml /etc/controlplane/config.toml

USER controlplane
ENV CONTROLPLANE_CONFIG=/etc/controlplane/config.toml
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/controlplane-server"]

