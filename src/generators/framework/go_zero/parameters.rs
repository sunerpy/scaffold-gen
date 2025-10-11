use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

use crate::constants::string_utils;
use crate::generators::core::Parameters;

#[derive(Debug, Clone)]
pub struct GoZeroParams {
    pub project_name: Option<String>,
    pub enable_api: bool,
    pub enable_rpc: bool,
    pub enable_admin: bool,
}

impl Default for GoZeroParams {
    fn default() -> Self {
        Self {
            project_name: None,
            enable_api: true,
            enable_rpc: false,
            enable_admin: false,
        }
    }
}

impl GoZeroParams {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn with_project_name(mut self, project_name: String) -> Self {
        self.project_name = Some(project_name);
        self
    }

    #[allow(dead_code)]
    pub fn with_api(mut self, enable_api: bool) -> Self {
        self.enable_api = enable_api;
        self
    }

    #[allow(dead_code)]
    pub fn with_rpc(mut self, enable_rpc: bool) -> Self {
        self.enable_rpc = enable_rpc;
        self
    }

    #[allow(dead_code)]
    pub fn with_admin(mut self, enable_admin: bool) -> Self {
        self.enable_admin = enable_admin;
        self
    }
}

impl Parameters for GoZeroParams {
    fn validate(&self) -> Result<()> {
        Ok(())
    }

    fn to_template_context(&self) -> HashMap<String, Value> {
        let mut context = HashMap::new();

        // Go-Zero specific parameters
        context.insert("enable_api".to_string(), Value::Bool(self.enable_api));
        context.insert("enable_rpc".to_string(), Value::Bool(self.enable_rpc));
        context.insert("enable_admin".to_string(), Value::Bool(self.enable_admin));

        // Project name and its variants
        if let Some(ref project_name) = self.project_name {
            context.insert(
                "project_name".to_string(),
                Value::String(project_name.clone()),
            );
            context.insert(
                "project_name_pascal".to_string(),
                Value::String(string_utils::to_pascal_case(project_name)),
            );
            context.insert(
                "project_name_snake".to_string(),
                Value::String(string_utils::to_snake_case(project_name)),
            );
        }

        context
    }
}
