#!/bin/bash

# Script to manage Airflow connections and variables

set -e

# Load environment variables
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

# Function to list all connections
list_connections() {
    echo "Listing all connections:"
    docker-compose exec -T airflow-webserver airflow connections list
}

# Function to get a specific connection
get_connection() {
    local conn_id=$1
    if [ -z "$conn_id" ]; then
        echo "Error: Connection ID is required"
        echo "Usage: $0 get-connection <connection_id>"
        exit 1
    fi
    
    echo "Getting connection: $conn_id"
    docker-compose exec -T airflow-webserver airflow connections get "$conn_id"
}

# Function to add or update a connection
set_connection() {
    local conn_id=$1
    local conn_type=$2
    local conn_uri=$3
    local description=${4:-}
    
    if [ -z "$conn_id" ] || [ -z "$conn_type" ] || [ -z "$conn_uri" ]; then
        echo "Error: Missing required parameters"
        echo "Usage: $0 set-connection <conn_id> <conn_type> <conn_uri> [description]"
        exit 1
    fi
    
    local cmd="airflow connections add '$conn_id' --conn-type '$conn_type' --conn-uri '$conn_uri'"
    
    if [ -n "$description" ]; then
        cmd+=" --conn-description '$description'"
    fi
    
    echo "Setting connection: $conn_id"
    docker-compose exec -T airflow-webserver bash -c "$cmd"
}

# Function to delete a connection
delete_connection() {
    local conn_id=$1
    if [ -z "$conn_id" ]; then
        echo "Error: Connection ID is required"
        echo "Usage: $0 delete-connection <connection_id>"
        exit 1
    fi
    
    read -p "Are you sure you want to delete connection '$conn_id'? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Deleting connection: $conn_id"
        docker-compose exec -T airflow-webserver airflow connections delete "$conn_id"
    else
        echo "Operation cancelled."
    fi
}

# Function to list all variables
list_variables() {
    echo "Listing all variables:"
    docker-compose exec -T airflow-webserver airflow variables list
}

# Function to get a specific variable
get_variable() {
    local var_name=$1
    if [ -z "$var_name" ]; then
        echo "Error: Variable name is required"
        echo "Usage: $0 get-variable <variable_name>"
        exit 1
    fi
    
    echo "Getting variable: $var_name"
    docker-compose exec -T airflow-webserver airflow variables get "$var_name"
}

# Function to set a variable
set_variable() {
    local var_name=$1
    local var_value=$2
    
    if [ -z "$var_name" ] || [ -z "$var_value" ]; then
        echo "Error: Variable name and value are required"
        echo "Usage: $0 set-variable <variable_name> <value>"
        exit 1
    fi
    
    echo "Setting variable: $var_name"
    docker-compose exec -T airflow-webserver airflow variables set "$var_name" "$var_value"
}

# Function to delete a variable
delete_variable() {
    local var_name=$1
    if [ -z "$var_name" ]; then
        echo "Error: Variable name is required"
        echo "Usage: $0 delete-variable <variable_name>"
        exit 1
    fi
    
    read -p "Are you sure you want to delete variable '$var_name'? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Deleting variable: $var_name"
        docker-compose exec -T airflow-webserver airflow variables delete "$var_name"
    else
        echo "Operation cancelled."
    fi
}

# Function to import variables from a JSON file
import_variables() {
    local json_file=$1
    
    if [ -z "$json_file" ] || [ ! -f "$json_file" ]; then
        echo "Error: JSON file is required and must exist"
        echo "Usage: $0 import-variables <path_to_json_file>"
        exit 1
    fi
    
    echo "Importing variables from $json_file"
    docker cp "$json_file" "$(docker-compose ps -q airflow-webserver):/tmp/variables.json"
    docker-compose exec -T airflow-webserver airflow variables import /tmp/variables.json
    docker-compose exec -T airflow-webserver rm /tmp/variables.json
    echo "Variables imported successfully!"
}

# Function to export variables to a JSON file
export_variables() {
    local json_file=${1:-/tmp/airflow_variables_$(date +%Y%m%d_%H%M%S).json}
    
    echo "Exporting variables to $json_file"
    docker-compose exec -T airflow-webserver airflow variables export /tmp/variables.json
    docker cp "$(docker-compose ps -q airflow-webserver):/tmp/variables.json" "$json_file"
    docker-compose exec -T airflow-webserver rm /tmp/variables.json
    echo "Variables exported to $json_file"
}

# Main script
case "$1" in
    # Connection commands
    list-connections)
        list_connections
        ;;
    get-connection)
        get_connection "$2"
        ;;
    set-connection)
        shift
        set_connection "$@"
        ;;
    delete-connection)
        delete_connection "$2"
        ;;
    
    # Variable commands
    list-variables)
        list_variables
        ;;
    get-variable)
        get_variable "$2"
        ;;
    set-variable)
        shift
        set_variable "$1" "$2"
        ;;
    delete-variable)
        delete_variable "$2"
        ;;
    import-variables)
        import_variables "$2"
        ;;
    export-variables)
        export_variables "$2"
        ;;
    *)
        echo "Usage: $0 {list-connections|get-connection|set-connection|delete-connection|list-variables|get-variable|set-variable|delete-variable|import-variables|export-variables}"
        echo "Connection Management:"
        echo "  list-connections                    List all connections"
        echo "  get-connection <conn_id>            Get connection details"
        echo "  set-connection <id> <type> <uri> [desc]  Add/update connection"
        echo "  delete-connection <conn_id>         Delete a connection"
        echo ""
        echo "Variable Management:"
        echo "  list-variables                      List all variables"
        echo "  get-variable <name>                 Get variable value"
        echo "  set-variable <name> <value>         Set a variable"
        echo "  delete-variable <name>              Delete a variable"
        echo "  import-variables <file.json>        Import variables from JSON file"
        echo "  export-variables [file.json]        Export variables to JSON file"
        exit 1
        ;;
esac
