"""
Configuration file for pytest.
This file contains fixtures and configuration for testing Airflow DAGs.
"""
import os
import pytest
from datetime import datetime
from unittest.mock import patch, MagicMock

from airflow import DAG
from airflow.models import TaskInstance, DagBag, Variable
from airflow.utils.state import State

# Set up test environment variables
os.environ['AIRFLOW_HOME'] = os.path.join(os.path.dirname(__file__), '..')
os.environ['ENVIRONMENT'] = 'test'
os.environ['K8S_NAMESPACE'] = 'test-namespace'
os.environ['MODEL_REGISTRY'] = 'test-registry'

@pytest.fixture(scope="module")
def test_dag():
    ""
    Fixture to create a test DAG instance.
    """
    return DAG(
        'test_dag',
        default_args={
            'owner': 'test',
            'start_date': datetime(2023, 1, 1),
        },
        schedule_interval='@daily',
    )

@pytest.fixture(scope="module")
def dag_bag():
    ""
    Fixture to load the DAG bag for testing.
    """
    return DagBag(dag_folder='dags/', include_examples=False)

@pytest.fixture(scope="function")
def mock_kubernetes_pod_operator():
    ""
    Fixture to mock the KubernetesPodOperator.
    """
    with patch('airflow.providers.cncf.kubernetes.operators.kubernetes_pod.KubernetesPodOperator') as mock:
        mock.return_value = MagicMock()
        yield mock

@pytest.fixture(scope="function")
def mock_variable():
    ""
    Fixture to mock Airflow Variables.
    """
    with patch('airflow.models.Variable') as mock:
        yield mock

@pytest.fixture(scope="function")
def mock_xcom():
    ""
    Fixture to mock XCom.
    """
    with patch('airflow.models.XCom') as mock:
        yield mock

@pytest.fixture(scope="function")
def mock_os_environ():
    ""
    Fixture to mock environment variables.
    ""
    with patch.dict('os.environ', {
        'ENVIRONMENT': 'test',
        'K8S_NAMESPACE': 'test-namespace',
        'MODEL_REGISTRY': 'test-registry'
    }):
        yield

@pytest.fixture(scope="function")
def mock_kubernetes_config():
    ""
    Fixture to mock Kubernetes config loading.
    """
    with patch('kubernetes.config.load_incluster_config'), \
         patch('kubernetes.config.load_kube_config'):
        yield

@pytest.fixture(scope="function")
def mock_kubernetes_client():
    ""
    Fixture to mock Kubernetes client.
    """
    with patch('kubernetes.client.CoreV1Api'), \
         patch('kubernetes.client.BatchV1Api'):
        yield

@pytest.fixture(scope="function")
def task_instance():
    ""
    Fixture to create a mock TaskInstance.
    """
    ti = MagicMock()
    ti.xcom_pull.return_value = 'test-version'
    return ti

@pytest.fixture(scope="function")
def context(task_instance):
    ""
    Fixture to create a context dictionary for task testing.
    """
    return {
        'ti': task_instance,
        'dag_run': MagicMock(),
        'execution_date': datetime(2023, 1, 1),
        'params': {},
        'conf': {}
    }
