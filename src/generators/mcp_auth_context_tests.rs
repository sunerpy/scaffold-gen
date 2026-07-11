use super::*;

#[test]
fn inject_into_inserts_all_five_keys() {
    let ctx = McpPythonAuthContext::new(McpBackend::Official, AuthMode::Jwt);
    let mut map = HashMap::new();
    ctx.inject_into(&mut map);

    assert_eq!(map.get("mcp_backend"), Some(&json!("official")));
    assert_eq!(map.get("mcp_backend_is_official"), Some(&json!(true)));
    assert_eq!(map.get("auth_mode"), Some(&json!("jwt")));
    assert_eq!(map.get("auth_enabled"), Some(&json!(true)));
    assert_eq!(map.get("auth_is_azure_ad"), Some(&json!(false)));
}

#[test]
fn inject_into_azure_ad_mode() {
    let ctx = McpPythonAuthContext::new(McpBackend::Fastmcp, AuthMode::AzureAd);
    let mut map = HashMap::new();
    ctx.inject_into(&mut map);

    assert_eq!(map.get("mcp_backend"), Some(&json!("fastmcp")));
    assert_eq!(map.get("mcp_backend_is_official"), Some(&json!(false)));
    assert_eq!(map.get("auth_mode"), Some(&json!("azure-ad")));
    assert_eq!(map.get("auth_enabled"), Some(&json!(true)));
    assert_eq!(map.get("auth_is_azure_ad"), Some(&json!(true)));
}

#[test]
fn inject_into_no_auth() {
    let ctx = McpPythonAuthContext::new(McpBackend::Official, AuthMode::None);
    let mut map = HashMap::new();
    ctx.inject_into(&mut map);

    assert_eq!(map.get("mcp_backend"), Some(&json!("official")));
    assert_eq!(map.get("mcp_backend_is_official"), Some(&json!(true)));
    assert_eq!(map.get("auth_mode"), Some(&json!("none")));
    assert_eq!(map.get("auth_enabled"), Some(&json!(false)));
    assert_eq!(map.get("auth_is_azure_ad"), Some(&json!(false)));
}
