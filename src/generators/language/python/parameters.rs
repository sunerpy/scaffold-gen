use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
            language_version: Some("3.11".to_string()),
            enable_modules: true,
            ..Default::default()
        };

        Self {
            base,
            uv_version: "0.9.1".to_string(),
            ruff_version: "0.12.1".to_string(),
        }
    }
}

impl InheritableParams for PythonParams {
    fn base_params(&self) -> &BaseParams {
        &self.base
    }

    fn base_params_mut(&mut self) -> &mut BaseParams {
        &mut self.base
    }

    fn from_base(base: BaseParams) -> Self {
        Self {
            base,
            uv_version: "0.9.1".to_string(),
            ruff_version: "0.12.1".to_string(),
        }
    }

    fn extended_template_context(&self) -> HashMap<String, Value> {
        let mut context = HashMap::new();

        // Python 特定的模板变量
        if let Some(ref version) = self.base.language_version {
            context.insert("python_version".to_string(), serde_json::json!(version));
        }

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
        base.language_version = Some("3.11".to_string());
        base.enable_modules = true;

        Self {
            base,
            uv_version: "0.9.1".to_string(),
            ruff_version: "0.12.1".to_string(),
        }
    }

    /// 从项目名称创建
    #[allow(dead_code)]
    pub fn from_project_name(project_name: String) -> Self {
        Self::new(project_name)
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

    /// 设置 ruff 版本
    #[allow(dead_code)]
    pub fn with_ruff_version(mut self, version: String) -> Self {
        self.ruff_version = version;
        self
    }

    /// 设置是否启用pre-commit
    pub fn with_precommit(mut self, enable: bool) -> Self {
        self.base.enable_precommit = enable;
        self
    }

    /// 设置许可证
    #[allow(dead_code)]
    pub fn with_license(mut self, license: String) -> Self {
        self.base.license = license;
        self
    }

    // 访问器方法
    #[allow(dead_code)]
    pub fn version(&self) -> Option<&String> {
        self.base.language_version.as_ref()
    }

    #[allow(dead_code)]
    pub fn package_name(&self) -> String {
        self.base
            .project_name
            .to_lowercase()
            .replace(['-', ' '], "_")
    }

    #[allow(dead_code)]
    pub fn enable_precommit(&self) -> bool {
        self.base.enable_precommit
    }
}
