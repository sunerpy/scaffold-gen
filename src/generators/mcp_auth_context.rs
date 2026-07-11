//! mcp-python auth template context — centralises the 5 auth-related keys injected
//! into the template rendering context after `to_template_context()`.
//!
//! Previously these insertions were scattered across `generate_mcp_python_language` in
//! the orchestrator and duplicated in integration test helpers. This struct provides a
//! single, reusable injection point for both production and test code.

use std::collections::HashMap;

use serde_json::{Value, json};

use crate::generators::auth_options::AuthMode;
use crate::generators::mcp_options::McpBackend;

/// mcp-python auth template context — centrally manages the 5 auth-related keys.
pub struct McpPythonAuthContext {
    pub backend: McpBackend,
    pub auth_mode: AuthMode,
}

impl McpPythonAuthContext {
    /// Create a new auth context from backend and auth mode.
    pub fn new(backend: McpBackend, auth_mode: AuthMode) -> Self {
        Self { backend, auth_mode }
    }

    /// Inject auth-related keys into the template context.
    pub fn inject_into(&self, context: &mut HashMap<String, Value>) {
        context.insert("mcp_backend".into(), json!(self.backend.as_str()));
        context.insert(
            "mcp_backend_is_official".into(),
            json!(self.backend.is_official()),
        );
        context.insert("auth_mode".into(), json!(self.auth_mode.as_str()));
        context.insert("auth_enabled".into(), json!(self.auth_mode.is_enabled()));
        context.insert(
            "auth_is_azure_ad".into(),
            json!(self.auth_mode.is_azure_ad()),
        );
    }
}

#[cfg(test)]
#[path = "mcp_auth_context_tests.rs"]
mod tests;
