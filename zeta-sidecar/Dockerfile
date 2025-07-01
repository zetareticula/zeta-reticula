FROM golang:1.22 AS builder

WORKDIR /app

COPY go.mod go.sum ./
RUN go mod download

COPY . .
RUN go build -o zeta-sidecar main.go

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/zeta-sidecar /usr/local/bin/zeta-sidecar

CMD ["zeta-sidecar"]

EXPOSE 50051

ENTRYPOINT ["/bin/sh", "-c", "zeta-sidecar"]

