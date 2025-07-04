package authz

default allow = false

allow {
    input.attributes.subscription_status == "active"
    input.attributes.subscription_plan >= input.resource_attrs.required_plan
}

# Example: Time-based restriction
allow {
    input.attributes.subscription_status == "active"
    input.attributes.subscription_plan >= input.resource_attrs.required_plan
    time.now_ns() < time.parse_rfc3339_ns(input.attributes.expires_at)
}