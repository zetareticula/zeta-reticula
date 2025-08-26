#!/bin/bash
set -e

# Create namespace for CI/CD tools
kubectl create namespace cicd --dry-run=client -o yaml | kubectl apply -f -

# Create service account for CI/CD
kubectl apply -f - <<EOF
apiVersion: v1
kind: ServiceAccount
metadata:
  name: github-actions
  namespace: production
  labels:
    app.kubernetes.io/name: github-actions
    app.kubernetes.io/component: ci-cd
---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: github-actions-role
  namespace: production
rules:
- apiGroups: [""]
  resources: ["pods", "pods/log"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["apps"]
  resources: ["deployments", "replicasets"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: ["autoscaling"]
  resources: ["horizontalpodautoscalers"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: github-actions-role-binding
  namespace: production
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: Role
  name: github-actions-role
subjects:
- kind: ServiceAccount
  name: github-actions
  namespace: production
EOF

# Get the service account token
SECRET_NAME=$(kubectl get serviceaccount github-actions -n production -o jsonpath='{.secrets[0].name}')
TOKEN=$(kubectl get secret $SECRET_NAME -n production -o jsonpath='{.data.token}' | base64 --decode)
CA_CRT=$(kubectl get secret $SECRET_NAME -n production -o jsonpath='{.data.ca\.crt}')

# Get the cluster endpoint
CLUSTER_ENDPOINT=$(kubectl config view --minify -o jsonpath='{.clusters[0].cluster.server}')
CLUSTER_NAME=$(kubectl config view --minify -o jsonpath='{.clusters[0].name}')

# Generate kubeconfig for GitHub Actions
cat > kubeconfig.yaml <<EOF
apiVersion: v1
kind: Config
clusters:
- name: ${CLUSTER_NAME}
  cluster:
    certificate-authority-data: ${CA_CRT}
    server: ${CLUSTER_ENDPOINT}
contexts:
- name: github-actions@${CLUSTER_NAME}
  context:
    cluster: ${CLUSTER_NAME}
    namespace: production
    user: github-actions
current-context: github-actions@${CLUSTER_NAME}
users:
- name: github-actions
  user:
    token: ${TOKEN}
EOF

echo "Kubernetes service account and RBAC configured for GitHub Actions"
echo "Kubeconfig has been saved to kubeconfig.yaml"
echo "Add this kubeconfig as a GitHub secret named 'KUBE_CONFIG'"
echo "Also add these GitHub secrets to your repository:"
echo "- DOCKERHUB_USERNAME: Your Docker Hub username"
echo "- DOCKERHUB_TOKEN: Your Docker Hub access token"
echo "- AWS_ACCESS_KEY_ID: AWS access key with EKS permissions"
echo "- AWS_SECRET_ACCESS_KEY: Corresponding AWS secret key"
