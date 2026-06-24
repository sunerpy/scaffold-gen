use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::generators::core::{BaseParams, InheritableParams};

/// Rust语言级别参数 - 继承自BaseParams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustParams {
    /// 基础参数
    pub base: BaseParams,
    /// Rust版本 (如 "1.75")
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
            language_version: Some("1.75".to_string()),
            ..Default::default()
        };

        Self {
            base,
            rust_version: Some("1.75".to_string()),
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

    fn base_params_mut(&mut self) -> &mut BaseParams {
        &mut self.base
    }

    fn from_base(base: BaseParams) -> Self {
        Self {
            base,
            rust_version: None,
            cargo_version: None,
            enable_proto_gen: false,
            enable_error_gen: false,
        }
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
            rust_version: Some("1.75".to_string()),
            cargo_version: None,
            enable_proto_gen: false,
            enable_error_gen: false,
        }
    }

    /// 设置Rust版本
    #[allow(dead_code)]
    pub fn with_rust_version(mut self, version: String) -> Self {
        self.rust_version = Some(version.clone());
        self.base.language_version = Some(version);
        self
    }

    /// 设置Cargo版本
    #[allow(dead_code)]
    pub fn with_cargo_version(mut self, version: String) -> Self {
        self.cargo_version = Some(version);
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

    /// 获取Rust版本
    #[allow(dead_code)]
    pub fn version(&self) -> Option<&String> {
        self.rust_version.as_ref()
    }

    /// 获取Cargo版本
    #[allow(dead_code)]
    pub fn get_cargo_version(&self) -> Option<&String> {
        self.cargo_version.as_ref()
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
