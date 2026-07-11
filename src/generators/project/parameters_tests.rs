use super::*;
use crate::generators::core::Parameters;

#[test]
fn new_sets_project_specific_defaults() {
    let params = ProjectParams::new("demo".to_string());
    assert_eq!(params.base.project_name, "demo");
    assert!(params.enable_git());
    assert!(!params.enable_precommit());
    assert_eq!(params.license(), "MIT");
    assert!(params.author().is_none());
}

#[test]
fn from_project_name_matches_new() {
    let a = ProjectParams::from_project_name("demo".to_string());
    let b = ProjectParams::new("demo".to_string());
    assert_eq!(a.base.project_name, b.base.project_name);
    assert_eq!(a.enable_git(), b.enable_git());
    assert_eq!(a.enable_precommit(), b.enable_precommit());
}

#[test]
fn default_has_empty_name_and_git_enabled_precommit_disabled() {
    let params = ProjectParams::default();
    assert_eq!(params.base.project_name, "");
    assert!(params.enable_git());
    assert!(!params.enable_precommit());
}

#[test]
fn with_description_sets_description() {
    let params = ProjectParams::new("demo".to_string()).with_description("a tool".to_string());
    assert_eq!(params.base.project_description, Some("a tool".to_string()));
}

#[test]
fn with_author_sets_author() {
    let params = ProjectParams::new("demo".to_string()).with_author("alice".to_string());
    assert_eq!(params.author(), &Some("alice".to_string()));
}

#[test]
fn with_license_sets_license() {
    let params = ProjectParams::new("demo".to_string()).with_license("GPL-3.0".to_string());
    assert_eq!(params.license(), "GPL-3.0");
}

#[test]
fn with_git_and_with_precommit_toggle_flags() {
    let params = ProjectParams::new("demo".to_string())
        .with_git(false)
        .with_precommit(true);
    assert!(!params.enable_git());
    assert!(params.enable_precommit());
}

#[test]
fn builders_chain_compose_all_fields() {
    let params = ProjectParams::new("my-proj".to_string())
        .with_description("desc".to_string())
        .with_author("bob".to_string())
        .with_license("Apache-2.0".to_string())
        .with_git(true)
        .with_precommit(true);

    assert_eq!(params.base.project_description, Some("desc".to_string()));
    assert_eq!(params.author(), &Some("bob".to_string()));
    assert_eq!(params.license(), "Apache-2.0");
    assert!(params.enable_git());
    assert!(params.enable_precommit());
}

#[test]
fn to_template_context_exposes_project_and_git_keys() {
    let params = ProjectParams::new("my-proj".to_string())
        .with_author("bob".to_string())
        .with_precommit(true);
    let ctx = params.to_template_context();

    assert_eq!(ctx["project_name"], serde_json::json!("my-proj"));
    assert_eq!(ctx["ProjectName"], serde_json::json!("my-proj"));
    assert_eq!(ctx["license"], serde_json::json!("MIT"));
    assert_eq!(ctx["author"], serde_json::json!("bob"));
    assert_eq!(ctx["enable_git"], serde_json::json!(true));
    assert_eq!(ctx["enable_precommit"], serde_json::json!(true));
}
