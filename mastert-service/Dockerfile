FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin master-service

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/master-service /usr/local/bin/
CMD ["master-service"]