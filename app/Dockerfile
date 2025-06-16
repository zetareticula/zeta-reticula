FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release
FROM debian:bullseye-slim
COPY --from=builder /app/target/release/billing-service /usr/local/bin/
CMD ["billing-service"]