package main

import (
	"context"
	"log"
	"net"

	"github.com/jackc/pgx/v5/pgxpool"
	pb "github.com/zetareticula/zeta-sidecar/pb/proto" // Generated protobuf package
	"github.com/zetareticula/zeta-sidecar/dsl"
	"google.golang.org/grpc"
)

const (
	parquetPath = "/cache/data.parquet"
	neonURL     = "postgres://user:password@ep-cool-name-123456.us-east-2.neon.tech/dbname?sslmode=require"
)

// Package main implements a gRPC server that provides caching and synchronization
// functionality for vector data and quantized layers. It supports in-memory caching,
// Parquet file storage, and synchronization with a Neon PostgreSQL database.
// It uses a DSL schema to define vector and layer structures, allowing for flexible
// configuration and extensibility. The server handles requests for cached data,
// updates to the cache, and synchronization with the database. The schema can be
// loaded from a JSON file, enabling easy modifications and additions of new vector types

type server struct {
	pb.SidecarServiceServer
	cache map[string][]byte // In-memory cache for vector data
}

func (s *server) GetCachedData(ctx context.Context, req *pb.CacheRequest) (*pb.CacheResponse, error) {
	key := req.VectorId + ":" + req.LayerId
	if data, ok := s.cache[key]; ok {
		return &pb.CacheResponse{Data: data, Status: "OK"}, nil
	}
	return &pb.CacheResponse{Status: "NOT_FOUND"}, nil
}

func (s *server) UpdateCache(ctx context.Context, req *pb.CacheUpdate) (*pb.UpdateResponse, error) {
	s.cache[req.VectorId] = req.Data
	saveToParquet(s.cache)
	syncWithNeon(req.VectorId, req.Data)
	return &pb.UpdateResponse{Status: "OK"}, nil
}

// saveToParquet is currently not implemented as it requires additional dependencies
// and proper schema definition for Parquet files.
// This function is kept as a placeholder for future implementation.
var saveToParquet = func(cache map[string][]byte) {
    log.Println("Parquet export not implemented")
}

func syncWithNeon(vectorID string, data []byte) {
    // Create a connection pool
    config, err := pgxpool.ParseConfig(neonURL)
    if err != nil {
        log.Fatalf("Unable to parse connection config: %v", err)
    }
    
    // Create a connection pool
    pool, err := pgxpool.NewWithConfig(context.Background(), config)
    if err != nil {
        log.Fatalf("Unable to create connection pool: %v", err)
    }
    defer pool.Close()

    // Execute the query
    _, err = pool.Exec(context.Background(), 
        "INSERT INTO cache (vector_id, data) VALUES ($1, $2) ON CONFLICT (vector_id) DO UPDATE SET data = $2", 
        vectorID, data)
    if err != nil {
        log.Fatalf("Error executing query: %v", err)
    }
}

func main() {
	lis, err := net.Listen("tcp", ":50051")
	if err != nil {
		log.Fatalf("failed to listen: %v", err)
	}

	s := grpc.NewServer()
	pb.RegisterSidecarServiceServer(s, &server{cache: make(map[string][]byte)})

	// Load DSL schema
	schema, err := dsl.LoadSchema("dsl/sample_config.json")
	if err != nil {
		log.Fatal(err)
	}
	log.Printf("Loaded schema with %d vectors and %d layers", len(schema.Vectors), len(schema.Layers))

	if err := s.Serve(lis); err != nil {
		log.Fatalf("failed to serve: %v", err)
	}
}
