FROM rust:1.97-bookworm AS builder
WORKDIR /source
COPY . .
RUN cargo build --locked --release -p ferrite-server

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /source/target/release/ferrite-server /usr/local/bin/ferrite-server
RUN useradd --system --uid 10001 --home-dir /var/lib/ferrite ferrite \
    && mkdir -p /etc/ferrite /var/lib/ferrite \
    && chown -R ferrite:ferrite /var/lib/ferrite
USER ferrite
EXPOSE 7000 7100 25565
ENTRYPOINT ["/usr/local/bin/ferrite-server"]
CMD ["--config", "/etc/ferrite/server.toml"]
