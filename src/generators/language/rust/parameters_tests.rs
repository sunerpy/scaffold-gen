use super::*;
use crate::generators::core::Parameters;

#[test]
fn new_sets_defaults() {
    let params = RustParams::new("demo".to_string());
    assert_eq!(params.base.project_name, "demo");
    assert_eq!(
        params.rust_version,
        Some(defaults::RUST_VERSION.to_string())
    );
    assert!(params.cargo_version.is_none());
    assert!(!params.enable_proto_gen());
    assert!(!params.enable_error_gen());
}

#[test]
fn default_carries_rust_version_into_base_language_version() {
    let params = RustParams::default();
    assert_eq!(
        params.rust_version,
        Some(defaults::RUST_VERSION.to_string())
    );
    assert_eq!(
        params.base.language_version,
        Some(defaults::RUST_VERSION.to_string())
    );
}

#[test]
fn with_rust_version_sets_both_fields() {
    let params = RustParams::new("demo".to_string()).with_rust_version("1.90".to_string());
    assert_eq!(params.rust_version, Some("1.90".to_string()));
    assert_eq!(params.base.language_version, Some("1.90".to_string()));
}

#[test]
fn with_proto_gen_and_error_gen_toggle() {
    let params = RustParams::new("demo".to_string())
        .with_proto_gen(true)
        .with_error_gen(true);
    assert!(params.enable_proto_gen());
    assert!(params.enable_error_gen());

    let off = RustParams::new("demo".to_string())
        .with_proto_gen(false)
        .with_error_gen(false);
    assert!(!off.enable_proto_gen());
    assert!(!off.enable_error_gen());
}

#[test]
fn to_template_context_includes_rust_version_and_gen_flags() {
    let params = RustParams::new("demo".to_string())
        .with_rust_version("1.89".to_string())
        .with_proto_gen(true)
        .with_error_gen(false);
    let ctx = params.to_template_context();

    assert_eq!(ctx["rust_version"], serde_json::json!("1.89"));
    assert_eq!(ctx["enable_proto_gen"], serde_json::json!(true));
    assert_eq!(ctx["enable_error_gen"], serde_json::json!(false));
    assert_eq!(ctx["language_version"], serde_json::json!("1.89"));
    assert_eq!(ctx["project_name"], serde_json::json!("demo"));
}

#[test]
fn to_template_context_omits_cargo_version_when_none_and_includes_when_set() {
    let default_ctx = RustParams::new("demo".to_string()).to_template_context();
    assert!(default_ctx.contains_key("rust_version"));

    let mut params = RustParams::new("demo".to_string());
    params.cargo_version = Some("1.88.0".to_string());
    let ctx = params.to_template_context();
    assert_eq!(ctx["cargo_version"], serde_json::json!("1.88.0"));
}

#[test]
fn to_template_context_omits_rust_version_key_when_none() {
    let mut params = RustParams::new("demo".to_string());
    params.rust_version = None;
    let ctx = params.to_template_context();
    assert!(!ctx.contains_key("rust_version"));
}
