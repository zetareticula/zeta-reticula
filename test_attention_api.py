#!/usr/bin/env python3
import requests
import time
import json
import uuid
import logging
import os
from typing import Dict, Any
from pathlib import Path

# Disable SSL warnings for self-signed certificates
import urllib3
urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class ZetaReticulaClient:
    def __init__(self, base_url: str = "https://localhost:8080/api", verify_cert: bool = False):
        """
        Initialize the Zeta Reticula API client.
        
        Args:
            base_url: Base URL of the API
            verify_cert: Whether to verify SSL certificates. Set to False for self-signed certs.
        """
        self.base_url = base_url
        self.session = requests.Session()
        self.session.verify = verify_cert
        if not verify_cert:
            # Add retry strategy for failed requests
            from requests.adapters import HTTPAdapter
            from requests.packages.urllib3.util.retry import Retry
            
            retry_strategy = Retry(
                total=3,
                backoff_factor=1,
                status_forcelist=[429, 500, 502, 503, 504],
                allowed_methods=["HEAD", "GET", "OPTIONS", "POST"]
            )
            adapter = HTTPAdapter(max_retries=retry_strategy)
            self.session.mount("https://", adapter)
            self.session.mount("http://", adapter)
            
        self.session.headers.update({
            "Content-Type": "application/json",
            "Accept": "application/json"
        })
        
    def authenticate(self, email: str, password: str) -> bool:
        """Authenticate and store the JWT token."""
        try:
            response = self.session.post(
                f"{self.base_url}/auth",
                json={"email": email, "password": password}
            )
            response.raise_for_status()
            token = response.json().get("token")
            if token:
                self.session.headers.update({"Authorization": f"Bearer {token}"})
                return True
        except Exception as e:
            logger.error(f"Authentication failed: {e}")
        return False
    
    def quantize_model(self, model_id: str, bit_width: int = 8, lora_enabled: bool = False) -> Dict[str, Any]:
        """Quantize a model using the attention-store and salience-engine."""
        try:
            response = self.session.post(
                f"{self.base_url}/quantize",
                json={
                    "model_id": model_id,
                    "bit_width": bit_width,
                    "lora_enabled": lora_enabled
                }
            )
            response.raise_for_status()
            return response.json()
        except Exception as e:
            logger.error(f"Quantization failed: {e}")
            return {"error": str(e)}
    
    def run_inference(self, model_id: str, prompt: str) -> Dict[str, Any]:
        """Run inference using the attention-store and track with agentflow."""
        try:
            response = self.session.post(
                f"{self.base_url}/inference/{model_id}",
                json={"prompt": prompt}
            )
            response.raise_for_status()
            return response.json()
        except Exception as e:
            logger.error(f"Inference failed: {e}")
            return {"error": str(e)}
    
    def get_agent_status(self) -> Dict[str, Any]:
        """Get the status of agentflow for the current session."""
        try:
            response = self.session.get(f"{self.base_url}/agent/status")
            response.raise_for_status()
            return response.json()
        except Exception as e:
            logger.error(f"Failed to get agent status: {e}")
            return {"error": str(e)}

def main():
    # Initialize client
    client = ZetaReticulaClient()
    
    # 1. Authenticate (using mock credentials for now)
    if not client.authenticate("test@example.com", "password123"):
        logger.error("Failed to authenticate")
        return
    
    # 2. Quantize a model (replace with actual model ID)
    model_id = "example-model"
    logger.info(f"Quantizing model {model_id}...")
    quantize_result = client.quantize_model(model_id, bit_width=8)
    logger.info(f"Quantization result: {json.dumps(quantize_result, indent=2)}")
    
    # 3. Run inference
    prompt = "Explain how attention mechanisms work in transformers"
    logger.info(f"Running inference with prompt: {prompt}")
    inference_result = client.run_inference(model_id, prompt)
    logger.info(f"Inference result: {json.dumps(inference_result, indent=2)}")
    
    # 4. Check agentflow status
    logger.info("Checking agentflow status...")
    agent_status = client.get_agent_status()
    logger.info(f"Agent status: {json.dumps(agent_status, indent=2)}")
    
    logger.info("Test completed!")

if __name__ == "__main__":
    main()
