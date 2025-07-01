module github.com/zetareticula/zeta-sidecar

go 1.23.0

toolchain go1.23.2

require (
	github.com/jackc/pgx/v5 v5.7.5 // Neon PostgreSQL driver
	google.golang.org/grpc v1.73.0 // gRPC
	google.golang.org/protobuf v1.36.6 // Protocol buffers
)

require (
	github.com/jackc/pgpassfile v1.0.0 // indirect
	github.com/jackc/pgservicefile v0.0.0-20240606120523-5a60cdf6a761 // indirect
	golang.org/x/crypto v0.37.0 // indirect
	golang.org/x/net v0.38.0 // indirect
	golang.org/x/sys v0.32.0 // indirect
	golang.org/x/text v0.24.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20250603155806-513f23925822 // indirect
)

require golang.org/x/sync v0.13.0 // indirect

require (
	github.com/jackc/puddle/v2 v2.2.2 // indirect
	github.com/stretchr/testify v1.9.0 // indirect
)
