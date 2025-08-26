terraform {
  required_version = ">= 1.0.0"
  
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 4.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.0"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.0"
    }
  }
  
  backend "s3" {
    # This will be configured when setting up the S3 backend
    # bucket         = "zeta-reticula-tfstate"
    # key            = "production/terraform.tfstate"
    # region         = "us-west-2"
    # dynamodb_table = "terraform-locks"
    # encrypt        = true
  }
}

# Provider configuration
provider "aws" {
  region = var.aws_region
  
  default_tags {
    tags = {
      Environment = var.environment
      Terraform   = "true"
      Project     = "zeta-reticula"
      Component   = "master-service"
    }
  }
}

# IAM Module
module "iam" {
  source = "../../modules/iam"
  
  environment = var.environment
  cluster_name = var.cluster_name
  
  tags = local.tags
}

# VPC Module
module "vpc" {
  source = "../../modules/vpc"
  
  environment = var.environment
  cluster_name = var.cluster_name
  
  vpc_cidr = var.vpc_cidr
  azs      = var.azs
  
  public_subnets  = var.public_subnets
  private_subnets = var.private_subnets
  
  tags = local.tags
}

# EKS Module
module "eks" {
  source = "../../modules/eks"
  
  environment = var.environment
  cluster_name = var.cluster_name
  
  aws_region = var.aws_region
  vpc_id     = module.vpc.vpc_id
  
  vpc_cidr = var.vpc_cidr
  azs      = var.azs
  
  public_subnets  = module.vpc.public_subnets
  private_subnets = module.vpc.private_subnets
  
  kubernetes_version = var.kubernetes_version
  
  desired_capacity = var.desired_capacity
  max_capacity     = var.max_capacity
  min_capacity     = var.min_capacity
  
  instance_types = var.instance_types
  capacity_type  = var.capacity_type
  disk_size      = var.disk_size
  
  map_roles = concat(
    [
      {
        rolearn  = "arn:aws:iam::${data.aws_caller_identity.current.account_id}:role/AWSReservedSSO_AdministratorAccess_1234567890abcdef"
        username = "admin"
        groups   = ["system:masters"]
      }
    ],
    var.map_roles
  )
  
  map_users = var.map_users
  
  tags = local.tags
}

# Kubernetes Provider
provider "kubernetes" {
  host                   = module.eks.cluster_endpoint
  cluster_ca_certificate = base64decode(module.eks.cluster_certificate_authority_data)
  
  exec {
    api_version = "client.authentication.k8s.io/v1beta1"
    command     = "aws"
    args = [
      "eks",
      "get-token",
      "--cluster-name",
      module.eks.cluster_name,
      "--region",
      var.aws_region
    ]
  }
}

# Helm Provider
provider "helm" {
  kubernetes {
    host                   = module.eks.cluster_endpoint
    cluster_ca_certificate = base64decode(module.eks.cluster_certificate_authority_data)
    
    exec {
      api_version = "client.authentication.k8s.io/v1beta1"
      command     = "aws"
      args = [
        "eks",
        "get-token",
        "--cluster-name",
        module.eks.cluster_name,
        "--region",
        var.aws_region
      ]
    }
  }
}

# AWS Load Balancer Controller IAM Role for Service Account
module "load_balancer_controller_irsa" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.0"
  
  role_name = "${var.cluster_name}-${var.environment}-load-balancer-controller"
  
  oidc_providers = {
    main = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:aws-load-balancer-controller"]
    }
  }
  
  role_policy_arns = {
    policy = module.iam.aws_load_balancer_controller_policy_arn
  }
  
  tags = local.tags
}

# EBS CSI Driver IAM Role for Service Account
module "ebs_csi_driver_irsa" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.0"
  
  role_name = "${var.cluster_name}-${var.environment}-ebs-csi-driver"
  
  oidc_providers = {
    main = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:ebs-csi-controller-sa"]
    }
  }
  
  role_policy_arns = {
    policy = module.iam.ebs_csi_driver_policy_arn
  }
  
  tags = local.tags
}

# Cluster Autoscaler IAM Role for Service Account
module "cluster_autoscaler_irsa" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.0"
  
  role_name = "${var.cluster_name}-${var.environment}-cluster-autoscaler"
  
  oidc_providers = {
    main = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:cluster-autoscaler"]
    }
  }
  
  role_policy_arns = {
    policy = module.iam.cluster_autoscaler_policy_arn
  }
  
  tags = local.tags
}

# External DNS IAM Role for Service Account
module "external_dns_irsa" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.0"
  
  role_name = "${var.cluster_name}-${var.environment}-external-dns"
  
  oidc_providers = {
    main = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:external-dns"]
    }
  }
  
  role_policy_arns = {
    policy = module.iam.external_dns_policy_arn
  }
  
  tags = local.tags
}

# Cert-Manager Module
module "cert_manager" {
  source = "../../modules/cert-manager"
  
  environment = var.environment
  cluster_name = var.cluster_name
  letsencrypt_email = var.letsencrypt_email
  
  tags = local.tags
  
  depends_on = [
    module.eks,
    module.external_dns_irsa
  ]
}

# Monitoring Module
module "monitoring" {
  source = "../../modules/monitoring"
  
  environment = var.environment
  cluster_name = var.cluster_name
  
  enable_loki = var.enable_loki
  
  prometheus_storage_class = var.prometheus_storage_class
  prometheus_storage_size = var.prometheus_storage_size
  prometheus_retention = var.prometheus_retention
  
  loki_storage_class = var.loki_storage_class
  loki_storage_size = var.loki_storage_size
  
  alertmanager_enabled = var.alertmanager_enabled
  grafana_admin_password = var.grafana_admin_password
  
  # Basic Auth Configuration
  basic_auth_enabled = true
  basic_auth_username = var.monitoring_auth_username
  basic_auth_password = var.monitoring_auth_password
  
  # Ingress Configuration
  ingress_enabled = true
  ingress_class_name = "nginx"
  tls_enabled = true
  
  tags = local.tags
  
  depends_on = [
    module.eks,
    module.cert_manager
  ]
}
