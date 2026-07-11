use super::*;

#[test]
fn parse_round_trip_for_both_backends() {
    for backend in [McpBackend::Fastmcp, McpBackend::Official] {
        let parsed = McpBackend::parse_from_str(backend.as_str());
        assert_eq!(parsed, Some(backend));
    }
}

#[test]
fn as_str_values() {
    assert_eq!(McpBackend::Fastmcp.as_str(), "fastmcp");
    assert_eq!(McpBackend::Official.as_str(), "official");
}

#[test]
fn is_official_flag() {
    assert!(McpBackend::Official.is_official());
    assert!(!McpBackend::Fastmcp.is_official());
}

#[test]
fn default_is_fastmcp() {
    assert_eq!(McpBackend::default(), McpBackend::Fastmcp);
}

#[test]
fn parse_accepts_mcp_alias_for_official() {
    assert_eq!(
        McpBackend::parse_from_str("mcp"),
        Some(McpBackend::Official)
    );
    assert_eq!(
        McpBackend::parse_from_str("MCP"),
        Some(McpBackend::Official)
    );
    assert_eq!(
        McpBackend::parse_from_str("FastMCP"),
        Some(McpBackend::Fastmcp)
    );
}

#[test]
fn parse_bogus_is_none() {
    assert!(McpBackend::parse_from_str("bogus").is_none());
}
