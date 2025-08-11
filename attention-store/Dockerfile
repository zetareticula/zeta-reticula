FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin attention-store

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/attention-store /usr/local/bin/
CMD ["attention-store"]