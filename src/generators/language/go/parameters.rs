use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::constants::defaults;
use crate::generators::core::{BaseParams, InheritableParams};

/// Go语言级别参数 - 现在继承自BaseParams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoParams {
    /// 基础参数
    pub base: BaseParams,
}

impl Default for GoParams {
    fn default() -> Self {
        let base = BaseParams {
            language_version: Some("1.21".to_string()),
            enable_modules: true,
            enable_cgo: false,
            enable_vendor: false,
            ..Default::default()
        };

        Self { base }
    }
}

impl InheritableParams for GoParams {
    fn base_params(&self) -> &BaseParams {
        &self.base
    }

    /// Go 工具链版本约束。挂在 Go 语言层而不是 `build_base_context`，是因为
    /// protoc-gen-jsonschema 只跟 Go 相关；挂在这里而不是 orchestrator 的 MCP 分支，
    /// 是因为生产渲染和测试 harness 都经由 `GoParams::to_template_context()`，
    /// 单点注入才能保证两边渲染出同一份内容。
    fn extended_template_context(&self) -> HashMap<String, Value> {
        let mut context = HashMap::new();
        context.insert(
            "protoc_gen_jsonschema_min_version".to_string(),
            json!(defaults::PROTOC_GEN_JSONSCHEMA_MIN_VERSION),
        );
        context
    }
}

impl GoParams {
    /// 创建新的Go参数
    pub fn new(module_name: String) -> Self {
        // 从模块名称中提取项目名称（取最后一部分）
        let project_name = module_name
            .split('/')
            .next_back()
            .unwrap_or(&module_name)
            .to_string();

        let mut base = BaseParams::new(project_name);

        // 设置Go特定的默认值
        base.language_version = Some("1.21".to_string());
        base.enable_modules = true;
        base.enable_cgo = false;
        base.enable_vendor = false;
        base.module_name = Some(module_name);

        Self { base }
    }

    /// 从项目名称创建
    pub fn from_project_name(project_name: String) -> Self {
        Self::new(Self::infer_module_name(&project_name))
    }

    /// 设置Go版本
    pub fn with_version(mut self, version: String) -> Self {
        self.base.language_version = Some(version);
        self
    }

    /// 从项目名称推断模块名称
    pub fn infer_module_name(project_name: &str) -> String {
        // 简单的模块名称推断逻辑
        format!(
            "github.com/example/{}",
            project_name.to_lowercase().replace(' ', "-")
        )
    }
}

#[cfg(test)]
#[path = "parameters_tests.rs"]
mod tests;
