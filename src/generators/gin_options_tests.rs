use super::*;

#[test]
fn new_and_default_have_all_none() {
    for opts in [GinProjectOptions::new(), GinProjectOptions::default()] {
        assert!(opts.description.is_none());
        assert!(opts.author.is_none());
        assert!(opts.license.is_none());
        assert!(opts.enable_git.is_none());
        assert!(opts.go_version.is_none());
        assert!(opts.module_name.is_none());
        assert!(opts.host.is_none());
        assert!(opts.port.is_none());
        assert!(opts.enable_swagger.is_none());
        assert!(opts.enable_cors.is_none());
        assert!(opts.enable_jwt.is_none());
        assert!(opts.enable_precommit.is_none());
        assert!(opts.enable_redis.is_none());
        assert!(opts.database_type.is_none());
    }
}

#[test]
fn with_license_sets_license() {
    let opts = GinProjectOptions::new().with_license("Apache-2.0".to_string());
    assert_eq!(opts.license, Some("Apache-2.0".to_string()));
    assert!(opts.host.is_none());
}

#[test]
fn with_server_sets_host_and_port() {
    let opts = GinProjectOptions::new().with_server("0.0.0.0".to_string(), 9090);
    assert_eq!(opts.host, Some("0.0.0.0".to_string()));
    assert_eq!(opts.port, Some(9090));
}

#[test]
fn with_swagger_toggles_flag() {
    assert_eq!(
        GinProjectOptions::new().with_swagger(true).enable_swagger,
        Some(true)
    );
    assert_eq!(
        GinProjectOptions::new().with_swagger(false).enable_swagger,
        Some(false)
    );
}

#[test]
fn with_precommit_toggles_flag() {
    assert_eq!(
        GinProjectOptions::new()
            .with_precommit(true)
            .enable_precommit,
        Some(true)
    );
    assert_eq!(
        GinProjectOptions::new()
            .with_precommit(false)
            .enable_precommit,
        Some(false)
    );
}

#[test]
fn builders_chain_and_compose() {
    let opts = GinProjectOptions::new()
        .with_license("MIT".to_string())
        .with_server("127.0.0.1".to_string(), 8080)
        .with_swagger(true)
        .with_precommit(false);

    assert_eq!(opts.license, Some("MIT".to_string()));
    assert_eq!(opts.host, Some("127.0.0.1".to_string()));
    assert_eq!(opts.port, Some(8080));
    assert_eq!(opts.enable_swagger, Some(true));
    assert_eq!(opts.enable_precommit, Some(false));
}
