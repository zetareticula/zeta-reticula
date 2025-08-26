# Terraform Configuration for Zeta Reticula Master Service

This directory contains the Terraform configuration for provisioning and managing the infrastructure for the Zeta Reticula Master Service on AWS EKS.

## Directory Structure

```
terraform/
├── modules/                 # Reusable Terraform modules
│   ├── eks/                 # EKS cluster module
│   ├── iam/                 # IAM roles and policies
│   └── vpc/                 # VPC and networking
└── environments/            # Environment-specific configurations
    ├── production/          # Production environment
    └── staging/             # Staging environment
```

## Prerequisites

1. [Terraform](https://www.terraform.io/downloads.html) >= 1.0.0
2. [AWS CLI](https://aws.amazon.com/cli/) configured with appropriate credentials
3. [kubectl](https://kubernetes.io/docs/tasks/tools/) for Kubernetes cluster interaction
4. [helm](https://helm.sh/docs/intro/install/) for managing Kubernetes packages

## Getting Started

### 1. Configure AWS Credentials

Make sure you have AWS credentials configured with sufficient permissions to create and manage EKS clusters.

```bash
export AWS_ACCESS_KEY_ID="your-access-key"
export AWS_SECRET_ACCESS_KEY="your-secret-key"
export AWS_DEFAULT_REGION="us-west-2"
```

### 2. Initialize Terraform

Navigate to the environment directory and initialize Terraform:

```bash
cd terraform/environments/production
terraform init
```

### 3. Review the Execution Plan

```bash
terraform plan
```

### 4. Apply the Configuration

```bash
terraform apply
```

This will create:
- A VPC with public and private subnets
- An EKS cluster with a managed node group
- IAM roles and policies
- OIDC provider for IAM Roles for Service Accounts (IRSA)

### 5. Configure kubectl

After the cluster is created, configure kubectl to connect to your cluster:

```bash
aws eks --region $(terraform output -raw aws_region) update-kubeconfig --name $(terraform output -raw cluster_name)
```

### 6. Verify Cluster Access

```bash
kubectl get nodes
kubectl get pods -A
```

## Managing the Infrastructure

### Updating the Cluster

To make changes to the infrastructure:

1. Modify the Terraform configuration files
2. Review the changes:
   ```bash
   terraform plan
   ```
3. Apply the changes:
   ```bash
   terraform apply
   ```

### Destroying the Infrastructure

To completely remove all created resources:

```bash
terraform destroy
```

> **Warning:** This will permanently delete all resources managed by Terraform.

## Modules

### EKS Module

Manages the EKS cluster, node groups, and related resources.

### IAM Module

Creates IAM roles and policies for the EKS cluster, node groups, and service accounts.

### VPC Module

Sets up the VPC, subnets, route tables, and NAT gateways for the EKS cluster.

## Best Practices

1. **Use workspaces or separate directories** for different environments (staging, production).
2. **Enable remote state** with locking using S3 and DynamoDB.
3. **Use variables** for all configurable parameters.
4. **Tag all resources** for better cost tracking and management.
5. **Enable EKS control plane logging** for security and debugging.

## Troubleshooting

### Common Issues

1. **Insufficient IAM Permissions**
   - Ensure your AWS user has the necessary permissions to create EKS clusters and related resources.
   - Use the `AdministratorAccess` policy for initial setup, then follow the principle of least privilege.

2. **VPC Quota Exceeded**
   - Check your AWS account's VPC quota and request an increase if needed.

3. **Node Group Failures**
   - Check the AWS EC2 console for any failed instances.
   - Review CloudWatch logs for error messages.

### Debugging

To enable debug logging for Terraform:

```bash
TF_LOG=DEBUG terraform apply
```

For Kubernetes-related issues:

```bash
kubectl describe <resource-type> <resource-name> -n <namespace>
kubectl logs <pod-name> -n <namespace>
```

## Security Considerations

1. **Encrypt EBS Volumes**: Enable encryption for EBS volumes used by the EKS nodes.
2. **Enable Encryption at Rest**: Use AWS KMS to encrypt Kubernetes secrets.
3. **Network Policies**: Implement network policies to control pod-to-pod communication.
4. **Pod Security Policies**: Use Pod Security Policies or OPA Gatekeeper for pod security.
5. **Regular Updates**: Keep the EKS control plane and worker nodes updated with the latest security patches.

## License

This Terraform configuration is part of the Zeta Reticula project and is licensed under the [MIT License](LICENSE).
