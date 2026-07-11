use super::*;
use crate::generators::core::Parameters;

#[test]
fn default_sets_gin_specific_flags() {
    let p = GinParams::default();
    assert_eq!(p.base.default_host, Some("127.0.0.1".to_string()));
    assert_eq!(p.base.default_port, Some(8080));
    assert!(p.base.enable_swagger);
    assert!(p.base.enable_cors);
    assert!(p.base.enable_middleware);
    assert!(p.base.enable_logging);
    assert_eq!(p.base.project_name, "");
}

#[test]
fn from_project_name_propagates_name_into_sub_params() {
    let p = GinParams::from_project_name("my-gin".to_string());
    assert_eq!(p.base.project_name, "my-gin");
    assert_eq!(p.base.default_host, Some("127.0.0.1".to_string()));
    assert_eq!(p.base.default_port, Some(8080));
    assert!(p.base.enable_swagger);
    assert!(p.base.enable_cors);
    assert!(p.base.enable_middleware);
    assert!(p.base.enable_logging);
    assert_eq!(
        p.go.base.module_name,
        Some("github.com/example/my-gin".to_string())
    );
}

#[test]
fn with_server_sets_host_and_port() {
    let p =
        GinParams::from_project_name("svc".to_string()).with_server("0.0.0.0".to_string(), 9090);
    assert_eq!(p.base.host, Some("0.0.0.0".to_string()));
    assert_eq!(p.base.port, Some(9090));
}

#[test]
fn with_swagger_and_accessor_round_trip() {
    let on = GinParams::from_project_name("s".to_string()).with_swagger(true);
    assert!(on.enable_swagger());
    let off = GinParams::from_project_name("s".to_string()).with_swagger(false);
    assert!(!off.enable_swagger());
}

#[test]
fn with_cors_toggles_flag() {
    let p = GinParams::from_project_name("s".to_string()).with_cors(false);
    assert!(!p.base.enable_cors);
}

#[test]
fn with_project_and_with_go_replace_sub_params() {
    let project = ProjectParams::from_project_name("proj".to_string());
    let go = GoParams::new("example.com/gomod".to_string());
    let p = GinParams::from_project_name("s".to_string())
        .with_project(project)
        .with_go(go);
    assert_eq!(p.project.base.project_name, "proj");
    assert_eq!(p.go.base.module_name, Some("example.com/gomod".to_string()));
}

#[test]
fn with_database_enables_flag_and_sets_type() {
    let p = GinParams::from_project_name("s".to_string()).with_database("postgres".to_string());
    assert!(p.base.enable_database);
    assert_eq!(p.base.database_type, Some("postgres".to_string()));
}

#[test]
fn with_redis_jwt_precommit_toggle_flags() {
    let p = GinParams::from_project_name("s".to_string())
        .with_redis(true)
        .with_jwt(true)
        .with_precommit(true);
    assert!(p.base.enable_redis);
    assert!(p.base.enable_jwt);
    assert!(p.enable_precommit());
}

#[test]
fn inheritable_params_expose_base_and_render_context() {
    let p = GinParams::from_project_name("ctx-app".to_string());
    assert_eq!(p.base_params().project_name, "ctx-app");
    let ctx = p.to_template_context();
    assert_eq!(ctx["project_name"], serde_json::json!("ctx-app"));
}
