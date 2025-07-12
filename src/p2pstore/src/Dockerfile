FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin p2pstore

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/p2pstore /usr/local/bin/
CMD ["p2pstore"]