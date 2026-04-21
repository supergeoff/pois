FROM rust:1.95-slim AS builder

WORKDIR /src

COPY . .

RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/target/release/pois /usr/local/bin/pois

ENV POIS_DATA_DIR=/data
VOLUME ["/data"]
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/pois", "gateway"]
