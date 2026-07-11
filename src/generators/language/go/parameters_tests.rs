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
    let params = GoParams::new("github.com/acme/svc".to_string()).with_version("1.24".to_string());
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
