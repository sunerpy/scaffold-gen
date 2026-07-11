use serde::{Deserialize, Serialize};

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
mod tests {
    use super::*;
    use crate::generators::core::Parameters;

    #[test]
    fn new_extracts_project_name_from_module_path() {
        let params = GoParams::new("github.com/acme/my-svc".to_string());
        assert_eq!(params.base.project_name, "my-svc");
        assert_eq!(
            params.base.module_name,
            Some("github.com/acme/my-svc".to_string())
        );
        assert_eq!(params.base.language_version, Some("1.21".to_string()));
        assert!(params.base.enable_modules);
        assert!(!params.base.enable_cgo);
        assert!(!params.base.enable_vendor);
    }

    #[test]
    fn new_without_slash_uses_whole_name() {
        let params = GoParams::new("standalone".to_string());
        assert_eq!(params.base.project_name, "standalone");
        assert_eq!(params.base.module_name, Some("standalone".to_string()));
    }

    #[test]
    fn default_sets_go_specific_flags() {
        let params = GoParams::default();
        assert_eq!(params.base.language_version, Some("1.21".to_string()));
        assert!(params.base.enable_modules);
        assert!(!params.base.enable_cgo);
        assert!(!params.base.enable_vendor);
    }

    #[test]
    fn infer_module_name_lowercases_and_hyphenates() {
        assert_eq!(
            GoParams::infer_module_name("My Cool App"),
            "github.com/example/my cool app".replace(' ', "-")
        );
        assert_eq!(
            GoParams::infer_module_name("My Cool App"),
            "github.com/example/my-cool-app"
        );
    }

    #[test]
    fn from_project_name_infers_module_and_extracts_name() {
        let params = GoParams::from_project_name("My App".to_string());
        assert_eq!(
            params.base.module_name,
            Some("github.com/example/my-app".to_string())
        );
        assert_eq!(params.base.project_name, "my-app");
    }

    #[test]
    fn with_version_overrides_language_version() {
        let params =
            GoParams::new("github.com/acme/svc".to_string()).with_version("1.24".to_string());
        assert_eq!(params.base.language_version, Some("1.24".to_string()));
    }

    #[test]
    fn to_template_context_carries_module_name_and_go_version() {
        let params =
            GoParams::new("github.com/acme/my-svc".to_string()).with_version("1.24".to_string());
        let ctx = params.to_template_context();

        assert_eq!(
            ctx["module_name"],
            serde_json::json!("github.com/acme/my-svc")
        );
        assert_eq!(ctx["go_version"], serde_json::json!("1.24"));
        assert_eq!(ctx["language_version"], serde_json::json!("1.24"));
        assert_eq!(ctx["project_name"], serde_json::json!("my-svc"));
        assert_eq!(ctx["enable_modules"], serde_json::json!(true));
    }
}
