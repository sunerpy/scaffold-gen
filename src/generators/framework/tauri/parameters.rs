use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
    /// 是否启用 error-gen 工具
    pub enable_error_gen: bool,
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
            enable_proto_gen: false,
            enable_error_gen: false,
        }
    }
}

impl InheritableParams for TauriParams {
    fn base_params(&self) -> &BaseParams {
        &self.base
    }

    fn extended_template_context(&self) -> HashMap<String, Value> {
        let mut context = HashMap::new();

        context.insert(
            "frontend_framework".to_string(),
            Value::String(self.frontend_framework.clone()),
        );
        context.insert(
            "enable_dark_mode".to_string(),
            Value::Bool(self.enable_dark_mode),
        );
        context.insert(
            "enable_skeleton".to_string(),
            Value::Bool(self.enable_skeleton),
        );
        context.insert(
            "window_width".to_string(),
            Value::Number(self.window_width.into()),
        );
        context.insert(
            "window_height".to_string(),
            Value::Number(self.window_height.into()),
        );
        context.insert(
            "identifier".to_string(),
            Value::String(self.identifier.clone()),
        );
        context.insert(
            "enable_proto_gen".to_string(),
            Value::Bool(self.enable_proto_gen),
        );
        context.insert(
            "enable_error_gen".to_string(),
            Value::Bool(self.enable_error_gen),
        );
        context.insert(
            "crate_name".to_string(),
            Value::String(crate::constants::string_utils::to_crate_ident(
                &self.base.project_name,
            )),
        );
        context.insert(
            "project_title".to_string(),
            Value::String(self.base.project_name.clone()),
        );

        context
    }
}

impl TauriParams {
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
            enable_proto_gen: false,
            enable_error_gen: false,
        }
    }

    /// 设置项目参数
    pub fn with_project(mut self, project: ProjectParams) -> Self {
        self.project = project;
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
    pub fn with_proto_gen(mut self, enable: bool) -> Self {
        self.enable_proto_gen = enable;
        self
    }

    /// 获取是否启用proto-gen工具
    pub fn enable_proto_gen(&self) -> bool {
        self.enable_proto_gen
    }

    pub fn with_error_gen(mut self, enable: bool) -> Self {
        self.enable_error_gen = enable;
        self
    }

    pub fn enable_error_gen(&self) -> bool {
        self.enable_error_gen
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::core::Parameters;

    #[test]
    fn default_sets_tauri_specific_fields() {
        let p = TauriParams::default();
        assert_eq!(p.base.default_host, Some("localhost".to_string()));
        assert_eq!(p.base.default_port, Some(1420));
        assert_eq!(p.frontend_framework, "vue");
        assert!(p.enable_dark_mode);
        assert!(p.enable_skeleton);
        assert_eq!(p.window_width, 800);
        assert_eq!(p.window_height, 600);
        assert_eq!(p.identifier, "com.example.app");
        assert!(!p.enable_proto_gen);
        assert!(!p.enable_error_gen);
    }

    #[test]
    fn from_project_name_derives_identifier_stripping_separators() {
        let p = TauriParams::from_project_name("My-Cool_App".to_string());
        assert_eq!(p.base.project_name, "My-Cool_App");
        assert_eq!(p.base.default_host, Some("localhost".to_string()));
        assert_eq!(p.base.default_port, Some(1420));
        assert_eq!(p.identifier, "com.mycoolapp.app");
        assert_eq!(p.rust.base.project_name, "My-Cool_App");
    }

    #[test]
    fn with_project_replaces_project_params() {
        let project = ProjectParams::from_project_name("proj".to_string());
        let p = TauriParams::from_project_name("s".to_string()).with_project(project);
        assert_eq!(p.project.base.project_name, "proj");
    }

    #[test]
    fn precommit_proto_error_builders_and_accessors_round_trip() {
        let p = TauriParams::from_project_name("s".to_string())
            .with_precommit(true)
            .with_proto_gen(true)
            .with_error_gen(true);
        assert!(p.enable_precommit());
        assert!(p.enable_proto_gen());
        assert!(p.enable_error_gen());
    }

    #[test]
    fn extended_template_context_carries_tauri_keys() {
        let p = TauriParams::from_project_name("app".to_string())
            .with_proto_gen(true)
            .with_error_gen(false);
        let ctx = p.extended_template_context();
        assert_eq!(ctx["frontend_framework"], Value::String("vue".to_string()));
        assert_eq!(ctx["enable_dark_mode"], Value::Bool(true));
        assert_eq!(ctx["enable_skeleton"], Value::Bool(true));
        assert_eq!(ctx["window_width"], Value::Number(800.into()));
        assert_eq!(ctx["window_height"], Value::Number(600.into()));
        assert_eq!(ctx["identifier"], Value::String("com.app.app".to_string()));
        assert_eq!(ctx["enable_proto_gen"], Value::Bool(true));
        assert_eq!(ctx["enable_error_gen"], Value::Bool(false));
        assert_eq!(ctx["crate_name"], Value::String("app".to_string()));
        assert_eq!(ctx["project_title"], Value::String("app".to_string()));
    }

    #[test]
    fn inheritable_params_merge_base_and_extended_context() {
        let p = TauriParams::from_project_name("merged".to_string());
        assert_eq!(p.base_params().project_name, "merged");
        let ctx = p.to_template_context();
        assert_eq!(ctx["project_name"], serde_json::json!("merged"));
        assert_eq!(ctx["frontend_framework"], serde_json::json!("vue"));
    }
}
