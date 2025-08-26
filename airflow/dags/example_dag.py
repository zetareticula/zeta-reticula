"""
Example DAG for testing Airflow setup.
"""
from datetime import datetime, timedelta
from airflow import DAG
from airflow.operators.python import PythonOperator
from airflow.operators.bash import BashOperator

default_args = {
    'owner': 'airflow',
    'depends_on_past': False,
    'email': ['your-email@example.com'],
    'email_on_failure': False,
    'email_on_retry': False,
    'retries': 1,
    'retry_delay': timedelta(minutes=5),
}

with DAG(
    'example_dag',
    default_args=default_args,
    description='A simple test DAG',
    schedule_interval=timedelta(days=1),
    start_date=datetime(2023, 1, 1),
    catchup=False,
    tags=['example'],
) as dag:

    # Task 1: Print a message
    print_hello = BashOperator(
        task_id='print_hello',
        bash_command='echo "Hello from Airflow!"',
    )

    # Task 2: Print the current timestamp
    def print_time():
        print(f"Current time: {datetime.now()}")

    print_time_task = PythonOperator(
        task_id='print_time',
        python_callable=print_time,
    )

    # Task 3: List files in the dags directory
    list_dags = BashOperator(
        task_id='list_dags',
        bash_command='ls -la /opt/airflow/dags',
    )

    # Set task dependencies
    print_hello >> print_time_task >> list_dags
