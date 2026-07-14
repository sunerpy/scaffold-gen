use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::constants::defaults;
use crate::generators::core::{BaseParams, InheritableParams};

/// Python语言级别参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonParams {
    /// 基础参数
    pub base: BaseParams,
    /// uv 版本
    pub uv_version: String,
    /// ruff 版本
    pub ruff_version: String,
}

impl Default for PythonParams {
    fn default() -> Self {
        let base = BaseParams {
            language_version: Some(defaults::PYTHON_MIN_VERSION.to_string()),
            enable_modules: true,
            ..Default::default()
        };

        Self {
            base,
            uv_version: defaults::UV_VERSION.to_string(),
            ruff_version: defaults::RUFF_VERSION.to_string(),
        }
    }
}

impl InheritableParams for PythonParams {
    fn base_params(&self) -> &BaseParams {
        &self.base
    }

    fn extended_template_context(&self) -> HashMap<String, Value> {
        let mut context = HashMap::new();

        // Python 特定的模板变量
        if let Some(ref version) = self.base.language_version {
            context.insert("python_version".to_string(), serde_json::json!(version));
        }
        context.insert(
            "python_min_version".to_string(),
            serde_json::json!(defaults::PYTHON_MIN_VERSION),
        );

        // 包名称（将项目名转换为有效的 Python 包名）
        let package_name = self
            .base
            .project_name
            .to_lowercase()
            .replace(['-', ' '], "_");
        context.insert("package_name".to_string(), serde_json::json!(package_name));

        // 工具版本
        context.insert("uv_version".to_string(), serde_json::json!(self.uv_version));
        context.insert(
            "ruff_version".to_string(),
            serde_json::json!(self.ruff_version),
        );

        context
    }
}

impl PythonParams {
    /// 创建新的Python参数
    pub fn new(project_name: String) -> Self {
        let mut base = BaseParams::new(project_name);

        // 设置Python特定的默认值
        base.language_version = Some(defaults::PYTHON_MIN_VERSION.to_string());
        base.enable_modules = true;

        Self {
            base,
            uv_version: defaults::UV_VERSION.to_string(),
            ruff_version: defaults::RUFF_VERSION.to_string(),
        }
    }

    /// 设置Python版本
    pub fn with_version(mut self, version: String) -> Self {
        self.base.language_version = Some(version);
        self
    }

    /// 设置 uv 版本
    pub fn with_uv_version(mut self, version: String) -> Self {
        self.uv_version = version;
        self
    }

    /// 设置是否启用pre-commit
    pub fn with_precommit(mut self, enable: bool) -> Self {
        self.base.enable_precommit = enable;
        self
    }
}

#[cfg(test)]
#[path = "parameters_tests.rs"]
mod tests;
