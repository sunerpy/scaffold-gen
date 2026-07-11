use super::*;

fn command_for(params: &ProjectParams) -> String {
    let cmd = NewCommand::new(params.project_path_file_name(), None);
    cmd.equivalent_command(params)
}

impl ProjectParams {
    fn project_path_file_name(&self) -> String {
        self.project_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    }
}

#[test]
fn python_fastapi_emits_network_and_no_rust_flags() {
    let params = ProjectParams {
        language: Language::Python,
        framework: Framework::FastApi,
        project_path: PathBuf::from("/tmp/work/testapi"),
        host: "0.0.0.0".to_string(),
        port: 8001,
        enable_precommit: true,
        license: "MIT".to_string(),
        enable_swagger: false,
        enable_proto_gen: false,
        enable_error_gen: false,
        enable_build: true,
        mcp_backend: McpBackend::Fastmcp,
        auth_mode: AuthMode::None,
    };

    assert_eq!(
        command_for(&params),
        "scafgen new testapi -p /tmp/work --language python --framework fastapi \
             --host 0.0.0.0 --port 8001 --precommit true --license MIT --with-build true"
    );
}

#[test]
fn rust_none_emits_framework_none_network_omitted_rust_tool_flags_present() {
    let params = ProjectParams {
        language: Language::Rust,
        framework: Framework::None,
        project_path: PathBuf::from("/tmp/work/mylib"),
        host: "0.0.0.0".to_string(),
        port: 8080,
        enable_precommit: false,
        license: "MIT".to_string(),
        enable_swagger: false,
        enable_proto_gen: false,
        enable_error_gen: false,
        enable_build: false,
        mcp_backend: McpBackend::Fastmcp,
        auth_mode: AuthMode::None,
    };

    let cmd = command_for(&params);
    assert_eq!(
        cmd,
        "scafgen new mylib -p /tmp/work --language rust --framework none --precommit false \
             --license MIT --proto-gen false --error-gen false --with-build false"
    );
    assert!(cmd.contains("--framework none"));
    assert!(!cmd.contains("--host"));
    assert!(!cmd.contains("--port"));
}

#[test]
fn go_gin_includes_swagger_and_network() {
    let params = ProjectParams {
        language: Language::Go,
        framework: Framework::Gin,
        project_path: PathBuf::from("/tmp/work/mygin"),
        host: "127.0.0.1".to_string(),
        port: 8080,
        enable_precommit: true,
        license: "Apache-2.0".to_string(),
        enable_swagger: true,
        enable_proto_gen: false,
        enable_error_gen: false,
        enable_build: false,
        mcp_backend: McpBackend::Fastmcp,
        auth_mode: AuthMode::None,
    };

    let cmd = command_for(&params);
    assert_eq!(
        cmd,
        "scafgen new mygin -p /tmp/work --language go --framework gin \
             --host 127.0.0.1 --port 8080 --precommit true --license Apache-2.0 \
             --swagger true --with-build false"
    );
    assert!(!cmd.contains("--proto-gen"));
}

#[test]
fn default_current_dir_path_omits_p_flag() {
    let params = ProjectParams {
        language: Language::Python,
        framework: Framework::None,
        project_path: PathBuf::from("./plain"),
        host: "0.0.0.0".to_string(),
        port: 8080,
        enable_precommit: true,
        license: "MIT".to_string(),
        enable_swagger: false,
        enable_proto_gen: false,
        enable_error_gen: false,
        enable_build: false,
        mcp_backend: McpBackend::Fastmcp,
        auth_mode: AuthMode::None,
    };

    let cmd = command_for(&params);
    assert_eq!(
        cmd,
        "scafgen new plain --language python --framework none --precommit true \
             --license MIT --with-build false"
    );
    assert!(!cmd.contains("-p "));
}

#[test]
fn python_mcp_python_emits_mcp_backend_flag() {
    let params = ProjectParams {
        language: Language::Python,
        framework: Framework::McpServerPython,
        project_path: PathBuf::from("/tmp/work/mcpsrv"),
        host: "0.0.0.0".to_string(),
        port: 8000,
        enable_precommit: false,
        license: "MIT".to_string(),
        enable_swagger: false,
        enable_proto_gen: false,
        enable_error_gen: false,
        enable_build: false,
        mcp_backend: McpBackend::Official,
        auth_mode: AuthMode::None,
    };

    let cmd = command_for(&params);
    assert_eq!(
        cmd,
        "scafgen new mcpsrv -p /tmp/work --language python --framework mcp-python \
             --host 0.0.0.0 --port 8000 --precommit false --license MIT \
             --with-build false --mcp-backend official --auth none"
    );
    assert!(cmd.contains("--mcp-backend official"));
    assert!(cmd.contains("--auth none"));
}

#[test]
fn python_mcp_python_jwt_emits_auth_flag() {
    let params = ProjectParams {
        language: Language::Python,
        framework: Framework::McpServerPython,
        project_path: PathBuf::from("/tmp/work/mcpauth"),
        host: "0.0.0.0".to_string(),
        port: 8000,
        enable_precommit: false,
        license: "MIT".to_string(),
        enable_swagger: false,
        enable_proto_gen: false,
        enable_error_gen: false,
        enable_build: false,
        mcp_backend: McpBackend::Fastmcp,
        auth_mode: AuthMode::Jwt,
    };

    let cmd = command_for(&params);
    assert_eq!(
        cmd,
        "scafgen new mcpauth -p /tmp/work --language python --framework mcp-python \
             --host 0.0.0.0 --port 8000 --precommit false --license MIT \
             --with-build false --mcp-backend fastmcp --auth jwt"
    );
    assert!(cmd.contains("--auth jwt"));
}

#[test]
fn non_mcp_python_never_emits_auth_flag() {
    let params = ProjectParams {
        language: Language::Python,
        framework: Framework::FastApi,
        project_path: PathBuf::from("/tmp/work/api"),
        host: "0.0.0.0".to_string(),
        port: 8000,
        enable_precommit: false,
        license: "MIT".to_string(),
        enable_swagger: false,
        enable_proto_gen: false,
        enable_error_gen: false,
        enable_build: false,
        mcp_backend: McpBackend::Fastmcp,
        auth_mode: AuthMode::Jwt,
    };

    let cmd = command_for(&params);
    assert!(!cmd.contains("--auth"));
}

#[test]
fn quote_if_needed_wraps_paths_with_spaces() {
    assert_eq!(quote_if_needed("MIT"), "MIT");
    assert_eq!(quote_if_needed("./my app"), "'./my app'");
    assert_eq!(quote_if_needed("plain/path"), "plain/path");
}

fn params_lf(language: Language, framework: Framework, auth_mode: AuthMode) -> ProjectParams {
    ProjectParams {
        language,
        framework,
        project_path: PathBuf::from("/tmp/work/proj"),
        host: "0.0.0.0".to_string(),
        port: 8000,
        enable_precommit: false,
        license: "MIT".to_string(),
        enable_swagger: false,
        enable_proto_gen: false,
        enable_error_gen: false,
        enable_build: false,
        mcp_backend: McpBackend::Fastmcp,
        auth_mode,
    }
}

#[test]
fn mcp_python_auth_none_emits_auth_none() {
    let params = params_lf(Language::Python, Framework::McpServerPython, AuthMode::None);
    let cmd = command_for(&params);
    assert!(
        cmd.contains("--auth none"),
        "mcp-python auth=none must emit `--auth none`, got: {cmd}"
    );
}

#[test]
fn mcp_python_auth_jwt_emits_auth_jwt() {
    let params = params_lf(Language::Python, Framework::McpServerPython, AuthMode::Jwt);
    let cmd = command_for(&params);
    assert!(
        cmd.contains("--auth jwt"),
        "mcp-python auth=jwt must emit `--auth jwt`, got: {cmd}"
    );
}

#[test]
fn mcp_python_auth_azure_ad_emits_auth_azure_ad() {
    let params = params_lf(
        Language::Python,
        Framework::McpServerPython,
        AuthMode::AzureAd,
    );
    let cmd = command_for(&params);
    assert!(
        cmd.contains("--auth azure-ad"),
        "mcp-python auth=azure-ad must emit `--auth azure-ad`, got: {cmd}"
    );
}

#[test]
fn python_none_emits_framework_none() {
    let params = params_lf(Language::Python, Framework::None, AuthMode::None);
    let cmd = command_for(&params);
    assert!(
        cmd.contains("--framework none"),
        "python framework=None must emit `--framework none`, got: {cmd}"
    );
}

#[test]
fn rust_none_emits_framework_none() {
    let params = params_lf(Language::Rust, Framework::None, AuthMode::None);
    let cmd = command_for(&params);
    assert!(
        cmd.contains("--framework none"),
        "rust framework=None must emit `--framework none`, got: {cmd}"
    );
}

#[test]
fn rust_tauri_emits_framework_tauri() {
    let params = params_lf(Language::Rust, Framework::Tauri, AuthMode::None);
    let cmd = command_for(&params);
    assert!(
        cmd.contains("--framework tauri"),
        "rust tauri must emit `--framework tauri`, got: {cmd}"
    );
}

#[test]
fn framework_flags_emitted_for_all_concrete_frameworks() {
    let cases = [
        (Language::Go, Framework::Gin, "--framework gin"),
        (Language::Go, Framework::McpServer, "--framework mcp-server"),
        (Language::Python, Framework::FastApi, "--framework fastapi"),
        (
            Language::Python,
            Framework::McpServerPython,
            "--framework mcp-python",
        ),
        (Language::Rust, Framework::Tauri, "--framework tauri"),
        (Language::TypeScript, Framework::Vue3, "--framework vue3"),
        (Language::TypeScript, Framework::React, "--framework react"),
    ];
    for (language, framework, expected) in cases {
        let params = params_lf(language, framework, AuthMode::None);
        let cmd = command_for(&params);
        assert!(
            cmd.contains(expected),
            "{language}/{framework:?} must emit `{expected}`, got: {cmd}"
        );
    }
}

#[test]
fn framework_none_parses_round_trip() {
    assert_eq!(Framework::parse_from_str("none"), Some(Framework::None));
}

#[test]
fn auth_none_parses_round_trip() {
    assert_eq!(AuthMode::parse_from_str("none"), Some(AuthMode::None));
}

#[test]
fn go_mcp_server_emits_network_and_no_swagger() {
    let params = ProjectParams {
        language: Language::Go,
        framework: Framework::McpServer,
        project_path: PathBuf::from("/tmp/work/mcpgo"),
        host: "0.0.0.0".to_string(),
        port: 8080,
        enable_precommit: true,
        license: "MIT".to_string(),
        enable_swagger: false,
        enable_proto_gen: false,
        enable_error_gen: false,
        enable_build: true,
        mcp_backend: McpBackend::Fastmcp,
        auth_mode: AuthMode::None,
    };

    let cmd = command_for(&params);
    assert_eq!(
        cmd,
        "scafgen new mcpgo -p /tmp/work --language go --framework mcp-server \
             --host 0.0.0.0 --port 8080 --precommit true --license MIT --with-build true"
    );
    assert!(cmd.contains("--host 0.0.0.0"));
    assert!(!cmd.contains("--swagger"));
    assert!(!cmd.contains("--auth"));
    assert!(!cmd.contains("--mcp-backend"));
}

#[test]
fn typescript_vue3_omits_network_and_rust_tool_flags() {
    let params = params_lf(Language::TypeScript, Framework::Vue3, AuthMode::None);
    let cmd = command_for(&params);
    assert_eq!(
        cmd,
        "scafgen new proj -p /tmp/work --language typescript --framework vue3 \
             --precommit false --license MIT --with-build false"
    );
    assert!(!cmd.contains("--host"));
    assert!(!cmd.contains("--port"));
    assert!(!cmd.contains("--proto-gen"));
    assert!(!cmd.contains("--swagger"));
}

#[test]
fn rust_tauri_emits_rust_tool_flags_no_network() {
    let params = ProjectParams {
        language: Language::Rust,
        framework: Framework::Tauri,
        project_path: PathBuf::from("/tmp/work/mytauri"),
        host: "0.0.0.0".to_string(),
        port: 8080,
        enable_precommit: false,
        license: "MIT".to_string(),
        enable_swagger: false,
        enable_proto_gen: true,
        enable_error_gen: true,
        enable_build: false,
        mcp_backend: McpBackend::Fastmcp,
        auth_mode: AuthMode::None,
    };

    let cmd = command_for(&params);
    assert_eq!(
        cmd,
        "scafgen new mytauri -p /tmp/work --language rust --framework tauri \
             --precommit false --license MIT --proto-gen true --error-gen true --with-build false"
    );
    assert!(!cmd.contains("--host"));
}

#[test]
fn parent_dir_with_spaces_is_quoted() {
    let params = ProjectParams {
        language: Language::Python,
        framework: Framework::None,
        project_path: PathBuf::from("/tmp/my work/proj"),
        host: "0.0.0.0".to_string(),
        port: 8080,
        enable_precommit: false,
        license: "MIT".to_string(),
        enable_swagger: false,
        enable_proto_gen: false,
        enable_error_gen: false,
        enable_build: false,
        mcp_backend: McpBackend::Fastmcp,
        auth_mode: AuthMode::None,
    };
    let cmd = command_for(&params);
    assert!(cmd.contains("-p '/tmp/my work'"));
}

#[test]
fn license_with_spaces_is_quoted() {
    let params = ProjectParams {
        language: Language::Python,
        framework: Framework::None,
        project_path: PathBuf::from("/tmp/work/proj"),
        host: "0.0.0.0".to_string(),
        port: 8080,
        enable_precommit: false,
        license: "BSD 3-Clause".to_string(),
        enable_swagger: false,
        enable_proto_gen: false,
        enable_error_gen: false,
        enable_build: false,
        mcp_backend: McpBackend::Fastmcp,
        auth_mode: AuthMode::None,
    };
    let cmd = command_for(&params);
    assert!(cmd.contains("--license 'BSD 3-Clause'"));
}

#[test]
fn quote_if_needed_wraps_empty_and_shell_metachars() {
    assert_eq!(quote_if_needed(""), "''");
    assert_eq!(quote_if_needed("a$b"), "'a$b'");
    assert_eq!(quote_if_needed("a`b"), "'a`b'");
    assert_eq!(quote_if_needed("a\"b"), "'a\"b'");
    assert_eq!(quote_if_needed("a\\b"), "'a\\b'");
}

#[test]
fn quote_if_needed_escapes_embedded_single_quote() {
    assert_eq!(quote_if_needed("it's"), r"'it'\''s'");
}

#[test]
fn resolve_mcp_backend_flag_parses_official() {
    let cmd = NewCommand::new("p".into(), None).with_mcp_backend(Some("official".into()));
    assert_eq!(
        cmd.resolve_mcp_backend(&Framework::McpServerPython)
            .unwrap(),
        McpBackend::Official
    );
}

#[test]
fn resolve_mcp_backend_flag_invalid_errors() {
    let cmd = NewCommand::new("p".into(), None).with_mcp_backend(Some("bogus".into()));
    let err = cmd
        .resolve_mcp_backend(&Framework::McpServerPython)
        .unwrap_err();
    assert!(err.to_string().contains("Unsupported mcp backend: bogus"));
}

#[test]
fn resolve_mcp_backend_non_mcp_default_is_fastmcp() {
    let cmd = NewCommand::new("p".into(), None);
    assert_eq!(
        cmd.resolve_mcp_backend(&Framework::FastApi).unwrap(),
        McpBackend::Fastmcp
    );
}

#[test]
fn resolve_auth_mode_flag_parses_jwt() {
    let cmd = NewCommand::new("p".into(), None).with_auth_mode(Some("jwt".into()));
    assert_eq!(
        cmd.resolve_auth_mode(&Framework::McpServerPython).unwrap(),
        AuthMode::Jwt
    );
}

#[test]
fn resolve_auth_mode_flag_parses_azure_ad() {
    let cmd = NewCommand::new("p".into(), None).with_auth_mode(Some("azure-ad".into()));
    assert_eq!(
        cmd.resolve_auth_mode(&Framework::McpServerPython).unwrap(),
        AuthMode::AzureAd
    );
}

#[test]
fn resolve_auth_mode_flag_invalid_errors() {
    let cmd = NewCommand::new("p".into(), None).with_auth_mode(Some("bogus".into()));
    let err = cmd
        .resolve_auth_mode(&Framework::McpServerPython)
        .unwrap_err();
    assert!(err.to_string().contains("Unsupported auth mode: bogus"));
}

#[test]
fn resolve_auth_mode_non_mcp_default_is_none() {
    let cmd = NewCommand::new("p".into(), None);
    assert_eq!(
        cmd.resolve_auth_mode(&Framework::FastApi).unwrap(),
        AuthMode::None
    );
}

#[test]
fn new_command_builders_set_all_fields() {
    let cmd = NewCommand::new("proj".into(), Some("/base".into()))
        .with_framework(Some("gin".into()))
        .with_host(Some("127.0.0.1".into()))
        .with_port(Some(9090))
        .with_grpc_port(Some(9091))
        .with_language(Some("go".into()))
        .with_precommit(Some(true))
        .with_license(Some("MIT".into()))
        .with_swagger(Some(true))
        .with_proto_gen(Some(false))
        .with_error_gen(Some(true))
        .with_build(Some(true))
        .with_mcp_backend(Some("fastmcp".into()))
        .with_auth_mode(Some("none".into()));

    assert_eq!(cmd.project_name, "proj");
    assert_eq!(cmd.target_path.as_deref(), Some("/base"));
    assert_eq!(cmd.framework.as_deref(), Some("gin"));
    assert_eq!(cmd.host.as_deref(), Some("127.0.0.1"));
    assert_eq!(cmd.port, Some(9090));
    assert_eq!(cmd.grpc_port, Some(9091));
    assert_eq!(cmd.language.as_deref(), Some("go"));
    assert_eq!(cmd.enable_precommit, Some(true));
    assert_eq!(cmd.license.as_deref(), Some("MIT"));
    assert_eq!(cmd.enable_swagger, Some(true));
    assert_eq!(cmd.enable_proto_gen, Some(false));
    assert_eq!(cmd.enable_error_gen, Some(true));
    assert_eq!(cmd.enable_build, Some(true));
    assert_eq!(cmd.mcp_backend.as_deref(), Some("fastmcp"));
    assert_eq!(cmd.auth_mode.as_deref(), Some("none"));
}

#[test]
fn new_command_defaults_are_all_none() {
    let cmd = NewCommand::new("proj".into(), None);
    assert!(cmd.target_path.is_none());
    assert!(cmd.framework.is_none());
    assert!(cmd.host.is_none());
    assert!(cmd.port.is_none());
    assert!(cmd.grpc_port.is_none());
    assert!(cmd.language.is_none());
    assert!(cmd.enable_precommit.is_none());
    assert!(cmd.license.is_none());
    assert!(cmd.enable_swagger.is_none());
    assert!(cmd.enable_proto_gen.is_none());
    assert!(cmd.enable_error_gen.is_none());
    assert!(cmd.enable_build.is_none());
    assert!(cmd.mcp_backend.is_none());
    assert!(cmd.auth_mode.is_none());
}

#[test]
fn project_name_with_spaces_is_quoted_in_command() {
    let params = params_lf(Language::Python, Framework::None, AuthMode::None);
    let cmd = NewCommand::new("my project".into(), None);
    let out = cmd.equivalent_command(&params);
    assert!(out.starts_with("scafgen new 'my project'"));
}
