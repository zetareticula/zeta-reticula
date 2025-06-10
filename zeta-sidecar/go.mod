module github.com/zetareticula/zeta-sidecar

go 1.22

require (
    github.com/apache/arrow/go/v12 v12.0.1 // Parquet support
    github.com/jackc/pgx/v5 v5.5.0        // Neon PostgreSQL driver
    google.golang.org/grpc v1.60.0         // gRPC
    google.golang.org/protobuf v1.31.0     // Protocol buffers
)