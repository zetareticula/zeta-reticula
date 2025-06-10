from airflow import DAG
from airflow.operators.python import PythonOperator
from airflow.providers.cncf.kubernetes.operators.kubernetes_pod import KubernetesPodOperator
from datetime import datetime, timedelta
import grpc
import zeta_sidecar_pb2
import zeta_sidecar_pb2_grpc
import psycopg2

default_args = {
    "owner": "airflow",
    "depends_on_past": False,
    "email_on_failure": False,
    "email_on_retry": False,
    "retries": 1,
    "retry_delay": timedelta(minutes=5),
}

with DAG(
    "zeta_reticula_workflow",
    default_args=default_args,
    description="Zeta Reticula Inference Workflow",
    schedule_interval=timedelta(hours=1),
    start_date=datetime(2025, 6, 9),
    catchup=False,
) as dag:

    def update_cache_task(**kwargs):
        channel = grpc.insecure_channel("zeta-sidecar:50051")
        stub = zeta_sidecar_pb2_grpc.SidecarServiceStub(channel)
        response = stub.UpdateCache(zeta_sidecar_pb2.CacheUpdate(
            vector_id="vec_001",
            data=b"cached_data"  # Replace with actual data
        ))
        print(response.status)

    def sync_with_neon_task(**kwargs):
        conn = psycopg2.connect(
            dbname="dbname",
            user="user",
            password="password",
            host="ep-cool-name-123456.us-east-2.neon.tech",
            port="5432",
            sslmode="require"
        )
        cur = conn.cursor()
        cur.execute("INSERT INTO cache (vector_id, data) VALUES (%s, %s) ON CONFLICT (vector_id) DO UPDATE SET data = %s",
                    ("vec_001", b"cached_data", b"cached_data"))
        conn.commit()
        cur.close()
        conn.close()

    ingest_model = KubernetesPodOperator(
        task_id="ingest_model",
        name="ingest-model",
        namespace="zeta-reticula",
        image="zetareticula/api:latest",
        cmds=["sh", "-c", "curl -F 'model=@model.bin' http://api:8080/upload"],
        in_cluster=True,
        get_logs=True,
    )

    quantize_model = KubernetesPodOperator(
        task_id="quantize_model",
        name="quantize-model",
        namespace="zeta-reticula",
        image="zetareticula/quantize-cli:latest",
        cmds=["sh", "-c", "quantize-cli --input model.bin --bits 8"],
        in_cluster=True,
        get_logs=True,
    )

    run_inference = KubernetesPodOperator(
        task_id="run_inference",
        name="run-inference",
        namespace="zeta-reticula",
        image="zetareticula/llm-rs:latest",
        cmds=["sh", "-c", "llm-rs --input 'sample text'"],
        in_cluster=True,
        get_logs=True,
    )

    update_cache = PythonOperator(
        task_id="update_cache",
        python_callable=update_cache_task,
    )

    sync_with_neon = PythonOperator(
        task_id="sync_with_neon",
        python_callable=sync_with_neon_task,
    )

    ingest_model >> quantize_model >> run_inference >> update_cache >> sync_with_neon