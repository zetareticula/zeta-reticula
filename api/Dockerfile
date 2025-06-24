FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y lua5.3 liblua5.3-dev python3 python3-dev
RUN cargo build --release --features enterprise

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y lua5.3 python3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zeta-reticula-api /usr/local/bin/
COPY lua/ /app/lua/
CMD ["zeta-reticula-api"]