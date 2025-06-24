function update_policy(user_id, plan)
    if plan == "enterprise" then
        return true, { resource_type = "all", required_plan = "enterprise", allow = true }
    end
    return false, {}
end