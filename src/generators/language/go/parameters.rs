use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::generators::core::Parameters;

/// Go语言级别参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoParams {
    /// Go版本
    pub version: String,
    /// Go模块名称
    pub module_name: String,
    /// 是否启用Go modules
    pub enable_modules: bool,
    /// 是否启用CGO
    pub enable_cgo: bool,
    /// Go构建标签
    pub build_tags: Vec<String>,
    /// 是否启用vendor
    pub enable_vendor: bool,
}

impl Default for GoParams {
    fn default() -> Self {
        Self {
            version: "1.21".to_string(),
            module_name: String::new(),
            enable_modules: true,
            enable_cgo: false,
            build_tags: Vec::new(),
            enable_vendor: false,
        }
    }
}

impl Parameters for GoParams {
    fn validate(&self) -> Result<()> {
        if self.module_name.is_empty() {
            return Err(anyhow::anyhow!("Go module name cannot be empty"));
        }

        // 验证Go版本格式
        if !self.version.chars().next().unwrap_or('0').is_ascii_digit() {
            return Err(anyhow::anyhow!(
                "Invalid Go version format: {}",
                self.version
            ));
        }

        Ok(())
    }

    fn to_template_context(&self) -> HashMap<String, Value> {
        let mut context = HashMap::new();

        context.insert("go_version".to_string(), json!(self.version));
        context.insert("module_name".to_string(), json!(self.module_name));
        context.insert("enable_modules".to_string(), json!(self.enable_modules));
        context.insert("enable_cgo".to_string(), json!(self.enable_cgo));
        context.insert("build_tags".to_string(), json!(self.build_tags));
        context.insert("enable_vendor".to_string(), json!(self.enable_vendor));

        // 添加Go相关的环境变量
        context.insert("goos".to_string(), json!(std::env::consts::OS));
        context.insert("goarch".to_string(), json!(std::env::consts::ARCH));

        context
    }

    fn override_from_env(&mut self) -> Result<()> {
        if let Ok(version) = std::env::var("GO_VERSION") {
            self.version = version;
        }

        if let Ok(module) = std::env::var("GO_MODULE") {
            self.module_name = module;
        }

        Ok(())
    }
}

impl GoParams {
    /// 创建新的Go参数
    pub fn new(module_name: String) -> Self {
        Self {
            module_name,
            ..Default::default()
        }
    }

    /// 设置Go版本
    pub fn with_version(mut self, version: String) -> Self {
        self.version = version;
        self
    }

    /// 启用CGO
    #[allow(dead_code)]
    pub fn with_cgo(mut self, enable: bool) -> Self {
        self.enable_cgo = enable;
        self
    }

    /// 添加构建标签
    #[allow(dead_code)]
    pub fn with_build_tag(mut self, tag: String) -> Self {
        self.build_tags.push(tag);
        self
    }

    /// 启用vendor
    #[allow(dead_code)]
    pub fn with_vendor(mut self, enable: bool) -> Self {
        self.enable_vendor = enable;
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
