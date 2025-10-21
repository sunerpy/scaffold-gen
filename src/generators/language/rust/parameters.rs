use serde::{Deserialize, Serialize};

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
        }
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
        }
    }

    /// 设置Rust版本
    pub fn with_rust_version(mut self, version: String) -> Self {
        self.rust_version = Some(version.clone());
        self.base.language_version = Some(version);
        self
    }

    /// 设置Cargo版本
    pub fn with_cargo_version(mut self, version: String) -> Self {
        self.cargo_version = Some(version);
        self
    }

    /// 获取Rust版本
    pub fn version(&self) -> Option<&String> {
        self.rust_version.as_ref()
    }

    /// 获取Cargo版本
    pub fn get_cargo_version(&self) -> Option<&String> {
        self.cargo_version.as_ref()
    }
}
