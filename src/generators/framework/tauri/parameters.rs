use serde::{Deserialize, Serialize};

use crate::generators::core::{BaseParams, InheritableParams};
use crate::generators::language::rust::RustParams;
use crate::generators::project::ProjectParams;

/// Tauri框架参数 - 继承自BaseParams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriParams {
    /// 基础参数
    pub base: BaseParams,
    /// 项目级别参数
    pub project: ProjectParams,
    /// Rust语言参数
    pub rust: RustParams,
    /// 前端框架类型 (vue, react, svelte, etc.)
    pub frontend_framework: String,
    /// 是否启用暗黑模式
    pub enable_dark_mode: bool,
    /// 是否启用骨架屏
    pub enable_skeleton: bool,
    /// 窗口宽度
    pub window_width: u32,
    /// 窗口高度
    pub window_height: u32,
    /// 应用标识符
    pub identifier: String,
    /// 是否启用 proto-gen 工具
    pub enable_proto_gen: bool,
}

impl Default for TauriParams {
    fn default() -> Self {
        let base = BaseParams {
            default_host: Some("localhost".to_string()),
            default_port: Some(1420),
            ..Default::default()
        };

        Self {
            base,
            project: ProjectParams::default(),
            rust: RustParams::default(),
            frontend_framework: "vue".to_string(),
            enable_dark_mode: true,
            enable_skeleton: true,
            window_width: 800,
            window_height: 600,
            identifier: "com.example.app".to_string(),
            enable_proto_gen: true,
        }
    }
}

impl InheritableParams for TauriParams {
    fn base_params(&self) -> &BaseParams {
        &self.base
    }

    fn base_params_mut(&mut self) -> &mut BaseParams {
        &mut self.base
    }

    fn from_base(base: BaseParams) -> Self {
        Self {
            base,
            project: ProjectParams::default(),
            rust: RustParams::default(),
            frontend_framework: "vue".to_string(),
            enable_dark_mode: true,
            enable_skeleton: true,
            window_width: 800,
            window_height: 600,
            identifier: "com.example.app".to_string(),
            enable_proto_gen: true,
        }
    }
}

impl TauriParams {
    /// 创建新的Tauri参数
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// 从项目名称创建
    pub fn from_project_name(project_name: String) -> Self {
        let mut base = BaseParams::new(project_name.clone());
        base.default_host = Some("localhost".to_string());
        base.default_port = Some(1420);

        let identifier = format!(
            "com.{}.app",
            project_name.to_lowercase().replace(['-', '_'], "")
        );

        Self {
            base,
            project: ProjectParams::from_project_name(project_name.clone()),
            rust: RustParams::new(project_name),
            frontend_framework: "vue".to_string(),
            enable_dark_mode: true,
            enable_skeleton: true,
            window_width: 800,
            window_height: 600,
            identifier,
            enable_proto_gen: true,
        }
    }

    /// 设置前端框架
    #[allow(dead_code)]
    pub fn with_frontend_framework(mut self, framework: String) -> Self {
        self.frontend_framework = framework;
        self
    }

    /// 设置是否启用暗黑模式
    #[allow(dead_code)]
    pub fn with_dark_mode(mut self, enable: bool) -> Self {
        self.enable_dark_mode = enable;
        self
    }

    /// 设置是否启用骨架屏
    #[allow(dead_code)]
    pub fn with_skeleton(mut self, enable: bool) -> Self {
        self.enable_skeleton = enable;
        self
    }

    /// 设置窗口尺寸
    #[allow(dead_code)]
    pub fn with_window_size(mut self, width: u32, height: u32) -> Self {
        self.window_width = width;
        self.window_height = height;
        self
    }

    /// 设置应用标识符
    #[allow(dead_code)]
    pub fn with_identifier(mut self, identifier: String) -> Self {
        self.identifier = identifier;
        self
    }

    /// 设置项目参数
    pub fn with_project(mut self, project: ProjectParams) -> Self {
        self.project = project;
        self
    }

    /// 设置Rust参数
    #[allow(dead_code)]
    pub fn with_rust(mut self, rust: RustParams) -> Self {
        self.rust = rust;
        self
    }

    /// 设置是否启用pre-commit
    pub fn with_precommit(mut self, enable_precommit: bool) -> Self {
        self.base.enable_precommit = enable_precommit;
        self
    }

    /// 获取是否启用pre-commit
    pub fn enable_precommit(&self) -> bool {
        self.base.enable_precommit
    }

    /// 设置是否启用proto-gen工具
    #[allow(dead_code)]
    pub fn with_proto_gen(mut self, enable: bool) -> Self {
        self.enable_proto_gen = enable;
        self
    }

    /// 获取是否启用proto-gen工具
    pub fn enable_proto_gen(&self) -> bool {
        self.enable_proto_gen
    }
}
