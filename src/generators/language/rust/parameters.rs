use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::constants::defaults;
use crate::generators::core::{BaseParams, InheritableParams};

/// Rust语言级别参数 - 继承自BaseParams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustParams {
    /// 基础参数
    pub base: BaseParams,
    /// Rust版本 (如 "1.88")
    pub rust_version: Option<String>,
    /// Cargo版本
    pub cargo_version: Option<String>,
    /// 是否启用 proto-gen 工具
    pub enable_proto_gen: bool,
    /// 是否启用 error-gen 工具
    pub enable_error_gen: bool,
}

impl Default for RustParams {
    fn default() -> Self {
        let base = BaseParams {
            language_version: Some(defaults::RUST_VERSION.to_string()),
            ..Default::default()
        };

        Self {
            base,
            rust_version: Some(defaults::RUST_VERSION.to_string()),
            cargo_version: None,
            enable_proto_gen: false,
            enable_error_gen: false,
        }
    }
}

impl InheritableParams for RustParams {
    fn base_params(&self) -> &BaseParams {
        &self.base
    }

    fn extended_template_context(&self) -> HashMap<String, Value> {
        let mut context = HashMap::new();

        if let Some(ref version) = self.rust_version {
            context.insert("rust_version".to_string(), Value::String(version.clone()));
        }

        if let Some(ref version) = self.cargo_version {
            context.insert("cargo_version".to_string(), Value::String(version.clone()));
        }

        context.insert(
            "enable_proto_gen".to_string(),
            Value::Bool(self.enable_proto_gen),
        );
        context.insert(
            "enable_error_gen".to_string(),
            Value::Bool(self.enable_error_gen),
        );

        context
    }
}

impl RustParams {
    /// 创建新的Rust参数
    pub fn new(project_name: String) -> Self {
        let base = BaseParams::new(project_name);

        Self {
            base,
            rust_version: Some(defaults::RUST_VERSION.to_string()),
            cargo_version: None,
            enable_proto_gen: false,
            enable_error_gen: false,
        }
    }

    /// 设置Rust版本
    pub fn with_rust_version(mut self, version: String) -> Self {
        self.rust_version = Some(version.clone());
        self.base.language_version = Some(version);
        self
    }

    /// 设置是否启用proto-gen工具
    pub fn with_proto_gen(mut self, enable: bool) -> Self {
        self.enable_proto_gen = enable;
        self
    }

    /// 设置是否启用error-gen工具
    pub fn with_error_gen(mut self, enable: bool) -> Self {
        self.enable_error_gen = enable;
        self
    }

    /// 获取是否启用proto-gen工具
    pub fn enable_proto_gen(&self) -> bool {
        self.enable_proto_gen
    }

    /// 获取是否启用error-gen工具
    pub fn enable_error_gen(&self) -> bool {
        self.enable_error_gen
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::core::Parameters;

    #[test]
    fn new_sets_defaults() {
        let params = RustParams::new("demo".to_string());
        assert_eq!(params.base.project_name, "demo");
        assert_eq!(
            params.rust_version,
            Some(defaults::RUST_VERSION.to_string())
        );
        assert!(params.cargo_version.is_none());
        assert!(!params.enable_proto_gen());
        assert!(!params.enable_error_gen());
    }

    #[test]
    fn default_carries_rust_version_into_base_language_version() {
        let params = RustParams::default();
        assert_eq!(
            params.rust_version,
            Some(defaults::RUST_VERSION.to_string())
        );
        assert_eq!(
            params.base.language_version,
            Some(defaults::RUST_VERSION.to_string())
        );
    }

    #[test]
    fn with_rust_version_sets_both_fields() {
        let params = RustParams::new("demo".to_string()).with_rust_version("1.90".to_string());
        assert_eq!(params.rust_version, Some("1.90".to_string()));
        assert_eq!(params.base.language_version, Some("1.90".to_string()));
    }

    #[test]
    fn with_proto_gen_and_error_gen_toggle() {
        let params = RustParams::new("demo".to_string())
            .with_proto_gen(true)
            .with_error_gen(true);
        assert!(params.enable_proto_gen());
        assert!(params.enable_error_gen());

        let off = RustParams::new("demo".to_string())
            .with_proto_gen(false)
            .with_error_gen(false);
        assert!(!off.enable_proto_gen());
        assert!(!off.enable_error_gen());
    }

    #[test]
    fn to_template_context_includes_rust_version_and_gen_flags() {
        let params = RustParams::new("demo".to_string())
            .with_rust_version("1.89".to_string())
            .with_proto_gen(true)
            .with_error_gen(false);
        let ctx = params.to_template_context();

        assert_eq!(ctx["rust_version"], serde_json::json!("1.89"));
        assert_eq!(ctx["enable_proto_gen"], serde_json::json!(true));
        assert_eq!(ctx["enable_error_gen"], serde_json::json!(false));
        assert_eq!(ctx["language_version"], serde_json::json!("1.89"));
        assert_eq!(ctx["project_name"], serde_json::json!("demo"));
    }

    #[test]
    fn to_template_context_omits_cargo_version_when_none_and_includes_when_set() {
        let default_ctx = RustParams::new("demo".to_string()).to_template_context();
        assert!(default_ctx.contains_key("rust_version"));

        let mut params = RustParams::new("demo".to_string());
        params.cargo_version = Some("1.88.0".to_string());
        let ctx = params.to_template_context();
        assert_eq!(ctx["cargo_version"], serde_json::json!("1.88.0"));
    }

    #[test]
    fn to_template_context_omits_rust_version_key_when_none() {
        let mut params = RustParams::new("demo".to_string());
        params.rust_version = None;
        let ctx = params.to_template_context();
        assert!(!ctx.contains_key("rust_version"));
    }
}
