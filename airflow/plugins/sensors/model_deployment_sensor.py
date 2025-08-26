""
Custom sensor for monitoring model deployments.
"""
import time
from typing import Dict, Optional, Sequence, Union

from airflow.exceptions import AirflowException
from airflow.providers.cncf.kubernetes.sensors.kubernetes_pod import KubernetesPodSensor
from kubernetes.client import models as k8s


class ModelDeploymentSensor(KubernetesPodSensor):
    """
    Custom sensor for monitoring model deployments.

    This sensor extends KubernetesPodSensor with additional functionality
    specific to model deployment monitoring, including:
    - Model version verification
    - Health check validation
    - Custom timeout and retry logic

    :param model_name: Name of the model being deployed
    :type model_name: str
    :param model_version: Version of the model being deployed
    :type model_version: str
    :param namespace: The kubernetes namespace where the pod is located
    :type namespace: str
    :param pod_name: The name of the pod to check
    :type pod_name: str
    :param timeout: Time in seconds to wait for the pod to be ready (default: 300)
    :type timeout: int
    :param retry_delay: Time in seconds to wait between retries (default: 30)
    :type retry_delay: int
    :param **kwargs: Additional arguments to pass to KubernetesPodSensor
    """

    template_fields: Sequence[str] = (
        "model_name",
        "model_version",
        "namespace",
        "pod_name",
    )

    def __init__(
        self,
        *,
        model_name: str,
        model_version: str,
        namespace: str,
        pod_name: str,
        timeout: int = 300,
        retry_delay: int = 30,
        **kwargs,
    ) -> None:
        self.model_name = model_name
        self.model_version = model_version
        self.namespace = namespace
        self.pod_name = pod_name
        self.timeout = timeout
        self.retry_delay = retry_delay

        # Call parent constructor
        super().__init__(
            namespace=namespace,
            pod_name=pod_name,
            **kwargs,
        )

    def poke(self, context) -> bool:
        """
        Check if the model deployment is complete.

        :param context: The context (same as other operators' context)
        :type context: dict
        :return: True if deployment is complete, False otherwise
        :rtype: bool
        """
        self.log.info(
            "Checking status of model %s (version: %s) deployment...",
            self.model_name,
            self.model_version,
        )

        try:
            # First check if the pod is ready using the parent's poke method
            is_ready = super().poke(context)
            
            if is_ready:
                # Additional checks specific to model deployment
                if self._check_model_health():
                    self.log.info(
                        "Model %s (version: %s) deployed successfully!",
                        self.model_name,
                        self.model_version,
                    )
                    return True
                
                self.log.info(
                    "Model %s (version: %s) pod is ready but health check failed. Retrying...",
                    self.model_name,
                    self.model_version,
                )
            
            return False
            
        except Exception as e:
            self.log.error(
                "Error checking model %s (version: %s) deployment: %s",
                self.model_name,
                self.model_version,
                str(e),
            )
            raise

    def _check_model_health(self) -> bool:
        """
        Perform additional health checks on the deployed model.
        
        This method can be overridden by subclasses to implement custom
        health check logic specific to the model being deployed.
        
        :return: True if the model is healthy, False otherwise
        :rtype: bool
        """
        # Default implementation just checks if the pod is ready
        # In a real implementation, you might want to:
        # 1. Make an HTTP request to the model's health endpoint
        # 2. Verify the model version matches what was deployed
        # 3. Run a smoke test prediction
        # 4. Check resource usage metrics
        return True

    def execute(self, context):
        """
        Execute the sensor.
        
        Overrides the parent method to add custom timeout handling.
        """
        started_at = time.time()
        
        while True:
            if self.poke(context):
                return True
                
            if (time.time() - started_at) > self.timeout:
                raise AirflowException(
                    f"Timeout: Model {self.model_name} (version: {self.model_version}) "
                    f"deployment did not complete within {self.timeout} seconds"
                )
                
            time.sleep(self.retry_delay)
