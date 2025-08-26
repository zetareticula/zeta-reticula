.PHONY: help build test run docker-build docker-run deploy-staging deploy-prod check-style check-format format clean

# Default target
help:
	@echo "Available targets:"
	@echo "  build         - Build the project"
	@echo "  test          - Run tests"
	@echo "  run           - Run the service locally"
	@echo "  docker-build  - Build Docker image"
	@echo "  docker-run    - Run Docker container"
	@echo "  deploy-staging - Deploy to staging environment"
	@echo "  deploy-prod   - Deploy to production environment"
	@echo "  check-style   - Check code style"
	@echo "  check-format  - Check code formatting"
	@echo "  format        - Format code"
	@echo "  clean         - Clean build artifacts"

# Build the project
build:
	cargo build --release

# Run tests
test:
	cargo test -- --nocapture

# Run the service locally
run:
	cargo run --release

# Build Docker image
docker-build:
	docker build -t zetareticula/master-service:latest .

# Run Docker container
docker-run:
	docker run -it --rm \
		-p 50051:50051 \
		-e RUST_LOG=info \
		-e BIND_ADDR=0.0.0.0:50051 \
		-e NODE_TIMEOUT_SECONDS=300 \
		zetareticula/master-service:latest

# Deploy to staging
deploy-staging:
	./deploy.sh --environment staging --tag latest-staging

# Deploy to production
deploy-prod:
	@read -p "Are you sure you want to deploy to production? (y/n) " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		read -p "Enter version tag (e.g., v1.0.0): " tag; \
		./deploy.sh --environment production --tag "$$tag"; \
	fi

# Check code style
check-style:
	cargo clippy -- -D warnings

# Check code formatting
check-format:
	cargo fmt -- --check

# Format code
format:
	cargo fmt

# Clean build artifacts
clean:
	cargo clean
	rm -f grpcurl

# Install development dependencies
install-deps:
	# Install protoc (Ubuntu/Debian)
	sudo apt-get update && sudo apt-get install -y protobuf-compiler
	# Install grpcurl
	if ! command -v grpcurl &> /dev/null; then \
		if [[ "$$OSTYPE" == "linux-gnu"* ]]; then \
			curl -sSL https://github.com/fullstorydev/grpcurl/releases/latest/download/grpcurl_$$(uname -m)_linux.tar.gz | tar xz; \
			sudo mv grpcurl /usr/local/bin/; \
		elif [[ "$$OSTYPE" == "darwin"* ]]; then \
			brew install grpcurl; \
		else \
			echo "Unsupported OS. Please install grpcurl manually."; \
			exit 1; \
		fi; \
	fi
