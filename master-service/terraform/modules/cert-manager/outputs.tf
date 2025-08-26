output "cert_manager_namespace" {
  description = "The namespace where cert-manager is installed"
  value       = kubernetes_namespace.cert_manager.metadata[0].name
}

output "letsencrypt_prod_issuer_name" {
  description = "Name of the Let's Encrypt production ClusterIssuer"
  value       = "letsencrypt-prod"
}

output "letsencrypt_staging_issuer_name" {
  description = "Name of the Let's Encrypt staging ClusterIssuer"
  value       = "letsencrypt-staging"
}
