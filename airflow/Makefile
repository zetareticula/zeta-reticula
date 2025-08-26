.PHONY: help setup test lint format clean start stop restart logs test-coverage docs

# Colors
RED=\033[0;31m
GREEN=\033[0;32m
YELLOW=\033[1;33m
NC=\033[0m # No Color

# Help
help:
	@echo "\n${YELLOW}Zeta Reticula Airflow - Development Tools${NC}\n"
	@echo "Available commands:\n"
	@echo "${GREEN}setup${NC}             - Set up the development environment"
	@echo "${GREEN}test${NC}              - Run tests"
	@echo "${GREEN}lint${NC}              - Run linters"
	@echo "${GREEN}format${NC}            - Format code"
	@echo "${GREEN}start${NC}             - Start Airflow services"
	@echo "${GREEN}stop${NC}              - Stop Airflow services"
	@echo "${GREEN}restart${NC}           - Restart Airflow services"
	@echo "${GREEN}logs${NC}              - View logs"
	@echo "${GREEN}clean${NC}             - Clean up temporary files"
	@echo "${GREEN}test-coverage${NC}     - Run tests with coverage report"
	@echo "${GREEN}docs${NC}              - Generate documentation"
	@echo ""

# Setup
dev-setup: setup
	@echo "\n${GREEN}✓ Development environment ready!${NC}\n"

setup:
	@echo "\n${YELLOW}Setting up development environment...${NC}"
	pip install -r requirements-test.txt
	test -f .env || cp .env.example .env
	@echo "\n${GREEN}✓ Setup complete! Edit .env file with your configuration.${NC}"

# Testing
test:
	@echo "\n${YELLOW}Running tests...${NC}"
	pytest tests/ -v

# Linting
lint:
	@echo "\n${YELLOW}Running linters...${NC}"
	@echo "\n${YELLOW}Running flake8...${NC}"
	flake8 dags/ tests/ --count --select=E9,F63,F7,F82 --show-source --statistics
	@echo "\n${YELLOW}Running mypy...${NC}"
	mypy dags/ tests/
	@echo "\n${GREEN}✓ Linting complete!${NC}"

# Formatting
format:
	@echo "\n${YELLOW}Formatting code...${NC}"
	black dags/ tests/
	isort dags/ tests/
	@echo "\n${GREEN}✓ Formatting complete!${NC}"

# Docker Compose commands
start:
	@echo "\n${YELLOW}Starting Airflow services...${NC}"
	docker-compose up -d
	@echo "\n${GREEN}✓ Airflow services started!${NC}\n"
	@echo "Airflow UI: http://localhost:8080"
	@echo "Username: admin"
	@echo "Password: admin"

stop:
	@echo "\n${YELLOW}Stopping Airflow services...${NC}"
	docker-compose down
	@echo "\n${GREEN}✓ Airflow services stopped.${NC}"

restart: stop start

logs:
	docker-compose logs -f

# Cleanup
clean:
	@echo "\n${YELLOW}Cleaning up...${NC}"
	find . -type d -name "__pycache__" -exec rm -r {} +
	find . -type d -name ".pytest_cache" -exec rm -r {} +
	rm -rf .coverage coverage.xml htmlcov/
	docker-compose down -v
	docker system prune -f
	@echo "\n${GREEN}✓ Cleanup complete!${NC}"

# Test coverage
test-coverage:
	@echo "\n${YELLOW}Running tests with coverage...${NC}"
	pytest --cov=dags --cov-report=term-missing --cov-report=xml:coverage.xml tests/
	@echo "\n${GREEN}✓ Coverage report generated!${NC}"

# Documentation
docs:
	@echo "\n${YELLOW}Generating documentation...${NC}"
	cd docs && make html
	@echo "\n${GREEN}✓ Documentation generated in docs/_build/html/${NC}"

# Default target
.DEFAULT_GOAL := help
