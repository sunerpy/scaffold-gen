use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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

    fn base_params_mut(&mut self) -> &mut BaseParams {
        &mut self.base
    }

    fn from_base(base: BaseParams) -> Self {
        Self { base }
    }

    // Go参数没有额外的参数，所有参数都在BaseParams中
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

    /// 启用CGO
    #[allow(dead_code)]
    pub fn with_cgo(mut self, enable: bool) -> Self {
        self.base.enable_cgo = enable;
        self
    }

    /// 添加构建标签
    #[allow(dead_code)]
    pub fn with_build_tag(mut self, tag: String) -> Self {
        self.base.build_tags.push(tag);
        self
    }

    /// 启用vendor
    #[allow(dead_code)]
    pub fn with_vendor(mut self, enable: bool) -> Self {
        self.base.enable_vendor = enable;
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

    // 为了向后兼容，提供访问器方法
    pub fn version(&self) -> Option<&String> {
        self.base.language_version.as_ref()
    }

    pub fn module_name(&self) -> Option<&String> {
        self.base.module_name.as_ref()
    }

    pub fn enable_modules(&self) -> bool {
        self.base.enable_modules
    }

    pub fn enable_cgo(&self) -> bool {
        self.base.enable_cgo
    }

    pub fn build_tags(&self) -> &Vec<String> {
        &self.base.build_tags
    }

    pub fn enable_vendor(&self) -> bool {
        self.base.enable_vendor
    }
}
