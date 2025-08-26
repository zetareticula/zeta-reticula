"""
Tests for the Zeta Reticula workflow DAG.
"""
import os
import unittest
from datetime import datetime
from unittest.mock import patch, MagicMock

from airflow import DAG
from airflow.models import TaskInstance, DagBag, Variable
from airflow.utils.state import State

# Add the parent directory to the Python path
import sys
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

# Import the DAG file
import dags.zeta_workflow as zeta_workflow

class TestZetaWorkflowDAG(unittest.TestCase):
    """Test cases for the Zeta Reticula workflow DAG."""
    
    @classmethod
    def setUpClass(cls):
        """Set up test environment before running tests."""
        # Set up test variables
        os.environ['ENVIRONMENT'] = 'test'
        os.environ['K8S_NAMESPACE'] = 'test-namespace'
        os.environ['MODEL_REGISTRY'] = 'test-registry'
        
        # Load the DAG
        cls.dagbag = DagBag(dag_folder='dags/', include_examples=False)
        cls.dag = cls.dagbag.get_dag('zeta_reticula_workflow')
    
    def test_dag_loaded(self):
        """Test that the DAG is properly loaded."""
        self.assertIsNotNone(self.dag)
        self.assertEqual(self.dag.dag_id, 'zeta_reticula_workflow')
    
    def test_dag_structure(self):
        """Test the structure of the DAG."""
        # Check the number of tasks
        self.assertEqual(len(self.dag.tasks), 8)  # 7 tasks + 1 start task
        
        # Check task dependencies
        task_names = [task.task_id for task in self.dag.tasks]
        self.assertIn('start_pipeline', task_names)
        self.assertIn('check_registry', task_names)
        self.assertIn('get_latest_model', task_names)
        self.assertIn('ingest_model', task_names)
        self.assertIn('quantize_model', task_names)
        self.assertIn('validate_model', task_names)
        self.assertIn('deploy_model', task_names)
        self.assertIn('test_model', task_names)
        self.assertIn('send_notification', task_names)
    
    @patch('dags.zeta_workflow.KubernetesPodOperator')
    def test_kubernetes_operators(self, mock_k8s_operator):
        """Test that KubernetesPodOperators are created with correct parameters."""
        # Mock the KubernetesPodOperator
        mock_operator = MagicMock()
        mock_k8s_operator.return_value = mock_operator
        
        # Re-import the module to apply the mock
        import importlib
        importlib.reload(zeta_workflow)
        
        # Check that the operators were created with the right parameters
        mock_k8s_operator.assert_any_call(
            task_id='ingest_model',
            name='ingest-model',
            namespace='test-namespace',
            image='test-registry/ingest:latest',
            cmds=['python', '-m', 'ingest.main', '--model-version', 'test-version'],
            env_vars={
                'LOG_LEVEL': 'INFO',
                'ENVIRONMENT': 'test',
                'MODEL_VERSION': 'test-version'
            },
            get_logs=True,
            in_cluster=True,
            is_delete_operator_pod=True,
            dag=self.dag
        )
    
    @patch('dags.zeta_workflow.Variable')
    def test_get_latest_model_task(self, mock_variable):
        """Test the get_latest_model task."""
        # Mock the Variable.get method
        mock_variable.get.return_value = 'test-version'
        
        # Call the task function
        context = {'ti': MagicMock()}
        result = zeta_workflow.get_latest_model(**context)
        
        # Check the result
        self.assertEqual(result, 'test-version')
        context['ti'].xcom_push.assert_called_once_with(key='model_version', value='test-version')
    
    def test_dag_schedule(self):
        """Test the DAG schedule."""
        self.assertEqual(self.dag.schedule_interval, '@daily')
    
    def test_dag_default_args(self):
        """Test the DAG default arguments."""
        default_args = self.dag.default_args
        self.assertEqual(default_args['owner'], 'zeta-team')
        self.assertEqual(default_args['start_date'], datetime(2023, 1, 1))
        self.assertEqual(default_args['email_on_failure'], True)
        self.assertEqual(default_args['email_on_retry'], False)
        self.assertEqual(default_args['retries'], 2)
        self.assertEqual(default_args['retry_delay'], 300)

if __name__ == '__main__':
    unittest.main()
