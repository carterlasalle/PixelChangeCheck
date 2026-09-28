# This image is primarily for `pcc relay` and `pcc view`. `pcc share` needs a
# display server, or `--synthetic` when a real display is unavailable.
FROM rust:1.85-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev libxcb1-dev libxrandr-dev libdbus-1-dev \
    libasound2-dev libopus-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
RUN cargo build --locked --release && strip target/release/pixel-change-check-client

FROM debian:stable-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libopus1 libasound2 libxcb1 libxrandr2 libdbus-1-3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --home-dir /var/lib/pcc --shell /usr/sbin/nologin pcc
COPY --from=builder /src/target/release/pixel-change-check-client /usr/local/bin/pcc

USER pcc
EXPOSE 5800 8080 5900
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD pcc --help >/dev/null || exit 1
ENTRYPOINT ["/usr/local/bin/pcc"]
