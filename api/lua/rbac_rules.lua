function generate_rbac_rules(plan)
    if plan == "enterprise" then
        return { { resource = "inference", allow = true }, { resource = "quantization", allow = true } }
    end
    return { { resource = "inference", allow = false }, { resource = "quantization", allow = false } }
end