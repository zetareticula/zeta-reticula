#!/bin/bash

# Database migration and maintenance script for Airflow

set -e

# Load environment variables
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

# Function to run migrations
run_migrations() {
    echo "Running database migrations..."
    docker-compose run --rm airflow-webserver airflow db upgrade
    echo "Migrations completed successfully!"
}

# Function to reset the database
reset_db() {
    read -p "This will delete all data in the Airflow database. Are you sure? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Resetting Airflow database..."
        docker-compose down -v
        docker volume rm airflow_postgres-db-volume
        docker-compose up -d postgres
        
        echo "Waiting for PostgreSQL to be ready..."
        until docker-compose exec -T postgres pg_isready -U airflow; do
            sleep 1
        done
        
        echo "Initializing Airflow database..."
        docker-compose run --rm airflow-webserver airflow db init
        docker-compose run --rm airflow-webserver airflow users create \
            --username admin \
            --firstname Admin \
            --lastname User \
            --role Admin \
            --email admin@example.com \
            --password admin
        
        echo "Database reset complete!"
    else
        echo "Database reset cancelled."
    fi
}

# Function to backup the database
backup_db() {
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local backup_file="airflow_db_backup_${timestamp}.sql"
    
    echo "Creating database backup to ${backup_file}..."
    docker-compose exec -T postgres pg_dump -U airflow airflow > "${backup_file}"
    
    if [ $? -eq 0 ]; then
        echo "Backup created successfully: ${backup_file}"
    else
        echo "Error creating backup!"
        exit 1
    fi
}

# Function to restore the database
restore_db() {
    local backup_file=$1
    
    if [ -z "${backup_file}" ]; then
        echo "Please specify a backup file to restore from."
        echo "Usage: $0 restore <backup_file.sql>"
        exit 1
    fi
    
    if [ ! -f "${backup_file}" ]; then
        echo "Backup file not found: ${backup_file}"
        exit 1
    fi
    
    read -p "This will overwrite the current database. Are you sure? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Restoring database from ${backup_file}..."
        docker-compose exec -T postgres psql -U airflow -d airflow < "${backup_file}"
        echo "Database restore complete!"
    else
        echo "Database restore cancelled."
    fi
}

# Main script
case "$1" in
    migrate)
        run_migrations
        ;;
    reset)
        reset_db
        ;;
    backup)
        backup_db
        ;;
    restore)
        restore_db "$2"
        ;;
    *)
        echo "Usage: $0 {migrate|reset|backup|restore}"
        echo "  migrate  - Run database migrations"
        echo "  reset    - Reset the database (DESTRUCTIVE)"
        echo "  backup   - Create a database backup"
        echo "  restore  - Restore from a backup"
        exit 1
        ;;
esac
