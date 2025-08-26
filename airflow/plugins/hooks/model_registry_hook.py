""
Custom hook for interacting with the model registry.
"""
import json
import logging
from typing import Dict, List, Optional, Union

import requests
from airflow.hooks.base import BaseHook
from requests.auth import HTTPBasicAuth


class ModelRegistryHook(BaseHook):
    """
    Interact with the model registry.

    This hook provides methods to interact with a model registry service,
    allowing you to register, update, and query model metadata.
    """

    conn_name_attr = 'model_registry_conn_id'
    default_conn_name = 'model_registry_default'
    conn_type = 'http'
    hook_name = 'Model Registry'

    def __init__(
        self,
        model_registry_conn_id: str = default_conn_name,
        *args,
        **kwargs,
    ) -> None:
        super().__init__(*args, **kwargs)
        self.conn_id = model_registry_conn_id
        self.base_url = None
        self.auth = None

    def get_conn(self) -> requests.Session:
        """
        Create and return a requests session with authentication.

        :return: A requests session with authentication
        :rtype: requests.Session
        """
        session = requests.Session()
        
        # Get connection details
        conn = self.get_connection(self.conn_id)
        
        # Set base URL
        schema = conn.schema or 'http'
        host = conn.host or 'localhost'
        port = conn.port or 8080
        self.base_url = f"{schema}://{host}:{port}/api/v1"
        
        # Set up authentication if credentials are provided
        if conn.login and conn.password:
            self.auth = HTTPBasicAuth(conn.login, conn.password)
        
        # Set up any additional headers
        if conn.extra:
            try:
                extra = json.loads(conn.extra)
                if 'headers' in extra and isinstance(extra['headers'], dict):
                    session.headers.update(extra['headers'])
            except json.JSONDecodeError:
                self.log.warning("Connection extra is not a valid JSON string")
        
        return session

    def get_model_versions(
        self,
        model_name: str,
        limit: int = 10,
        offset: int = 0,
    ) -> List[Dict]:
        """
        Get a list of versions for a specific model.

        :param model_name: Name of the model
        :type model_name: str
        :param limit: Maximum number of versions to return
        :type limit: int
        :param offset: Number of versions to skip
        :type offset: int
        :return: List of model versions
        :rtype: List[Dict]
        """
        session = self.get_conn()
        url = f"{self.base_url}/models/{model_name}/versions"
        params = {"limit": limit, "offset": offset}
        
        try:
            response = session.get(url, params=params, auth=self.auth)
            response.raise_for_status()
            return response.json().get('versions', [])
        except requests.exceptions.RequestException as e:
            self.log.error(f"Failed to get model versions: {str(e)}")
            raise

    def get_latest_version(self, model_name: str) -> Optional[Dict]:
        """
        Get the latest version of a model.

        :param model_name: Name of the model
        :type model_name: str
        :return: Latest model version or None if not found
        :rtype: Optional[Dict]
        """
        versions = self.get_model_versions(model_name, limit=1)
        return versions[0] if versions else None

    def register_model_version(
        self,
        model_name: str,
        version: str,
        description: str = "",
        metadata: Optional[Dict] = None,
    ) -> Dict:
        """
        Register a new version of a model.

        :param model_name: Name of the model
        :type model_name: str
        :param version: Version string (e.g., '1.0.0')
        :type version: str
        :param description: Description of the model version
        :type description: str
        :param metadata: Additional metadata for the model version
        :type metadata: Dict, optional
        :return: The created model version
        :rtype: Dict
        """
        session = self.get_conn()
        url = f"{self.base_url}/models/{model_name}/versions"
        
        payload = {
            "version": version,
            "description": description,
            "metadata": metadata or {},
        }
        
        try:
            response = session.post(
                url,
                json=payload,
                auth=self.auth,
                headers={"Content-Type": "application/json"},
            )
            response.raise_for_status()
            return response.json()
        except requests.exceptions.RequestException as e:
            self.log.error(f"Failed to register model version: {str(e)}")
            raise

    def update_model_version(
        self,
        model_name: str,
        version: str,
        description: str = None,
        metadata: Optional[Dict] = None,
    ) -> Dict:
        """
        Update an existing model version.

        :param model_name: Name of the model
        :type model_name: str
        :param version: Version string to update
        :type version: str
        :param description: New description (optional)
        :type description: str, optional
        :param metadata: Metadata to update (will be merged with existing)
        :type metadata: Dict, optional
        :return: The updated model version
        :rtype: Dict
        """
        session = self.get_conn()
        url = f"{self.base_url}/models/{model_name}/versions/{version}"
        
        # Get current version first to merge metadata
        current = self.get_model_version(model_name, version)
        
        payload = {}
        if description is not None:
            payload["description"] = description
        
        if metadata is not None:
            current_metadata = current.get("metadata", {})
            current_metadata.update(metadata)
            payload["metadata"] = current_metadata
        
        try:
            response = session.patch(
                url,
                json=payload,
                auth=self.auth,
                headers={"Content-Type": "application/json"},
            )
            response.raise_for_status()
            return response.json()
        except requests.exceptions.RequestException as e:
            self.log.error(f"Failed to update model version: {str(e)}")
            raise

    def get_model_version(
        self,
        model_name: str,
        version: str,
    ) -> Dict:
        """
        Get details for a specific model version.

        :param model_name: Name of the model
        :type model_name: str
        :param version: Version string
        :type version: str
        :return: Model version details
        :rtype: Dict
        :raises: AirflowException if the model version is not found
        """
        session = self.get_conn()
        url = f"{self.base_url}/models/{model_name}/versions/{version}"
        
        try:
            response = session.get(url, auth=self.auth)
            response.raise_for_status()
            return response.json()
        except requests.exceptions.HTTPError as e:
            if e.response.status_code == 404:
                raise AirflowException(
                    f"Model version {model_name}:{version} not found"
                ) from e
            raise
        except requests.exceptions.RequestException as e:
            self.log.error(f"Failed to get model version: {str(e)}")
            raise

    def set_model_stage(
        self,
        model_name: str,
        version: str,
        stage: str,
        description: str = "",
    ) -> Dict:
        """
        Set the stage of a model version (e.g., 'staging', 'production').

        :param model_name: Name of the model
        :type model_name: str
        :param version: Version string
        :type version: str
        :param stage: Target stage (e.g., 'staging', 'production')
        :type stage: str
        :param description: Description of the stage transition
        :type description: str
        :return: The updated model version
        :rtype: Dict
        """
        session = self.get_conn()
        url = f"{self.base_url}/model-versions/set-stage"
        
        payload = {
            "name": model_name,
            "version": version,
            "stage": stage,
            "description": description,
        }
        
        try:
            response = session.post(
                url,
                json=payload,
                auth=self.auth,
                headers={"Content-Type": "application/json"},
            )
            response.raise_for_status()
            return response.json()
        except requests.exceptions.RequestException as e:
            self.log.error(f"Failed to set model stage: {str(e)}")
            raise
