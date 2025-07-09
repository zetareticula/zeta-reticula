#!/bin/bash

# Check Minikube status
echo "=== Minikube Status ==="
minikube status

echo "\n=== Kubernetes Version ==="
kubectl version --short

echo "\n=== Current Context ==="
kubectl config current-context

echo "\n=== Available Namespaces ==="
kubectl get namespaces

echo "\n=== All Resources ==="
kubectl get all

echo "\n=== Pods Status ==="
kubectl get pods -o wide

echo "\n=== Pod Events ==="
kubectl get events --sort-by='.metadata.creationTimestamp'

echo "\n=== Services ==="
kubectl get svc

echo "\n=== Deployments ==="
kubectl get deployments

echo "\n=== Helm Charts ==="
helm list

echo "\n=== Docker Images ==="
docker images | grep zetareticula

echo "\n=== Minikube Docker Env ==="
eval $(minikube docker-env)
docker images | grep zetareticula
