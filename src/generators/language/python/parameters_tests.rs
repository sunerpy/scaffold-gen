use super::*;
use crate::generators::core::Parameters;

#[test]
fn new_sets_python_defaults() {
    let params = PythonParams::new("demo".to_string());
    assert_eq!(params.base.project_name, "demo");
    assert_eq!(
        params.base.language_version,
        Some(defaults::PYTHON_MIN_VERSION.to_string())
    );
    assert!(params.base.enable_modules);
    assert_eq!(params.uv_version, defaults::UV_VERSION.to_string());
    assert_eq!(params.ruff_version, defaults::RUFF_VERSION.to_string());
}

#[test]
fn default_matches_min_version_and_tool_versions() {
    let params = PythonParams::default();
    assert_eq!(
        params.base.language_version,
        Some(defaults::PYTHON_MIN_VERSION.to_string())
    );
    assert!(params.base.enable_modules);
    assert_eq!(params.uv_version, defaults::UV_VERSION.to_string());
    assert_eq!(params.ruff_version, defaults::RUFF_VERSION.to_string());
}

#[test]
fn with_version_sets_language_version() {
    let params = PythonParams::new("demo".to_string()).with_version("3.13".to_string());
    assert_eq!(params.base.language_version, Some("3.13".to_string()));
}

#[test]
fn with_uv_version_sets_uv_version() {
    let params = PythonParams::new("demo".to_string()).with_uv_version("0.9.5".to_string());
    assert_eq!(params.uv_version, "0.9.5".to_string());
}

#[test]
fn with_precommit_toggles_flag() {
    assert!(
        PythonParams::new("demo".to_string())
            .with_precommit(true)
            .base
            .enable_precommit
    );
    assert!(
        !PythonParams::new("demo".to_string())
            .with_precommit(false)
            .base
            .enable_precommit
    );
}

#[test]
fn to_template_context_exposes_python_version_package_name_and_tools() {
    let params = PythonParams::new("My-Cool App".to_string())
        .with_version("3.13".to_string())
        .with_uv_version("0.9.5".to_string());
    let ctx = params.to_template_context();

    assert_eq!(ctx["python_version"], serde_json::json!("3.13"));
    assert_eq!(ctx["package_name"], serde_json::json!("my_cool_app"));
    assert_eq!(ctx["uv_version"], serde_json::json!("0.9.5"));
    assert_eq!(
        ctx["ruff_version"],
        serde_json::json!(defaults::RUFF_VERSION)
    );
    assert_eq!(ctx["project_name"], serde_json::json!("My-Cool App"));
}

#[test]
fn template_context_keeps_host_and_minimum_python_versions_distinct() {
    let params = PythonParams::new("demo".to_string()).with_version("3.14".to_string());

    let ctx = params.to_template_context();

    assert_eq!(ctx["python_version"], serde_json::json!("3.14"));
    assert_eq!(
        ctx["python_min_version"],
        serde_json::json!(defaults::PYTHON_MIN_VERSION)
    );
    assert_ne!(ctx["python_version"], ctx["python_min_version"]);
}

#[test]
fn package_name_normalizes_hyphen_space_and_case() {
    let params = PythonParams::new("Foo-Bar Baz".to_string());
    let ctx = params.to_template_context();
    assert_eq!(ctx["package_name"], serde_json::json!("foo_bar_baz"));
}
