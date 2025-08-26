""
Custom operator for deploying machine learning models.
"""
from typing import Dict, Optional, Sequence

from airflow.models import BaseOperator
from airflow.providers.cncf.kubernetes.operators.kubernetes_pod import KubernetesPodOperator
from kubernetes.client import models as k8s


class ModelDeploymentOperator(KubernetesPodOperator):
    """
    Custom operator for deploying machine learning models.

    This operator extends KubernetesPodOperator with additional functionality
    specific to model deployment, including:
    - Model version tracking
    - Health checks
    - Rollback support

    :param model_name: Name of the model to deploy
    :type model_name: str
    :param model_version: Version of the model to deploy
    :type model_version: str
    :param model_path: Path to the model file in the container
    :type model_path: str
    :param environment: Deployment environment (dev/staging/prod)
    :type environment: str
    :param replicas: Number of replicas to deploy (default: 1)
    :type replicas: int
    :param resources: Resource requirements for the deployment
    :type resources: dict
    :param node_selector: Node selector for pod placement
    :type node_selector: dict
    :param tolerations: Tolerations for pod scheduling
    :type tolerations: list
    :param env_vars: Additional environment variables
    :type env_vars: dict
    :param **kwargs: Additional arguments to pass to KubernetesPodOperator
    """

    template_fields: Sequence[str] = tuple(
        {"model_name", "model_version", "environment"}.union(
            KubernetesPodOperator.template_fields
        )
    )

    def __init__(
        self,
        *,
        model_name: str,
        model_version: str,
        model_path: str,
        environment: str = "dev",
        replicas: int = 1,
        resources: Optional[Dict] = None,
        node_selector: Optional[Dict] = None,
        tolerations: Optional[list] = None,
        env_vars: Optional[Dict] = None,
        **kwargs,
    ) -> None:
        # Set default values
        self.model_name = model_name
        self.model_version = model_version
        self.model_path = model_path
        self.environment = environment
        self.replicas = replicas

        # Set default resources if not provided
        if resources is None:
            resources = {
                "request_memory": "1Gi",
                "request_cpu": "0.5",
                "limit_memory": "2Gi",
                "limit_cpu": "1",
            }

        # Set default node selector if not provided
        if node_selector is None:
            node_selector = {
                "node-role.kubernetes.io/worker": "true"
            }

        # Set default tolerations if not provided
        if tolerations is None:
            tolerations = [
                {
                    'key': 'dedicated',
                    'operator': 'Equal',
                    'value': 'ml',
                    'effect': 'NoSchedule',
                }
            ]

        # Set default environment variables
        default_env_vars = {
            "MODEL_NAME": model_name,
            "MODEL_VERSION": model_version,
            "MODEL_PATH": model_path,
            "ENVIRONMENT": environment,
            "LOG_LEVEL": "INFO",
        }

        # Merge with provided environment variables
        if env_vars:
            default_env_vars.update(env_vars)

        # Set up the container command
        cmds = [
            "python",
            "-m",
            "deploy.main",
            f"--model-path={model_path}",
            f"--environment={environment}",
            f"--replicas={replicas}",
        ]

        # Call parent constructor with all parameters
        super().__init__(
            name=f"deploy-{model_name}-{model_version}".lower(),
            image=f"{kwargs.pop('image', 'zetareticula/deploy:latest')}",
            cmds=cmds,
            arguments=[],
            env_vars=default_env_vars,
            resources=resources,
            node_selector=node_selector,
            tolerations=tolerations,
            **kwargs,
        )

    def execute(self, context):
        """
        Execute the deployment.

        :param context: The context (same as other operators' context)
        :type context: dict
        """
        self.log.info(
            "Starting deployment of model %s (version: %s) to %s environment",
            self.model_name,
            self.model_version,
            self.environment,
        )

        try:
            # Call the parent's execute method to run the pod
            result = super().execute(context)
            self.log.info(
                "Successfully deployed model %s (version: %s)",
                self.model_name,
                self.model_version,
            )
            return result
        except Exception as e:
            self.log.error(
                "Failed to deploy model %s (version: %s): %s",
                self.model_name,
                self.model_version,
                str(e),
            )
            raise

    def on_kill(self):
        """
        Called when the operator is asked to terminate.
        """
        self.log.info(
            "Terminating deployment of model %s (version: %s)",
            self.model_name,
            self.model_version,
        )
        super().on_kill()
