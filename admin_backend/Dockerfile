FROM rust:1.83-bookworm AS builder

WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim

RUN useradd --system --create-home --home-dir /home/keylesspass keylesspass \
    && mkdir -p /data \
    && chown -R keylesspass:keylesspass /data

COPY --from=builder /app/target/release/keylesspass-admin /usr/local/bin/keylesspass-admin

USER keylesspass
ENV KEYLESSPASS_ADMIN_BIND=0.0.0.0:8787
ENV KEYLESSPASS_ADMIN_DB=/data/keylesspass-admin.sqlite3
EXPOSE 8787
VOLUME ["/data"]

CMD ["keylesspass-admin"]
