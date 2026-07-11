use super::*;

/// 注册表中每个框架的语言归属，必须与 `frameworks_for_language` 一致。
#[test]
fn registry_language_matches_frameworks_for_language() {
    for spec in REGISTRY {
        let langs = [
            Language::Go,
            Language::Python,
            Language::Rust,
            Language::TypeScript,
        ];
        let owner = langs
            .into_iter()
            .find(|l| Framework::frameworks_for_language(*l).contains(&spec.framework))
            .unwrap_or_else(|| {
                panic!(
                    "framework {:?} not listed in any frameworks_for_language",
                    spec.framework
                )
            });
        assert_eq!(
            owner, spec.language,
            "registry language for {:?} disagrees with frameworks_for_language",
            spec.framework
        );
    }
}

/// `frameworks_for_language` 列出的每个非 None 框架都能被解析出规格。
#[test]
fn every_listed_framework_resolves() {
    let langs = [
        Language::Go,
        Language::Python,
        Language::Rust,
        Language::TypeScript,
    ];
    for lang in langs {
        for fw in Framework::frameworks_for_language(lang) {
            if fw == Framework::None {
                continue;
            }
            let spec = resolve(lang, fw)
                .unwrap_or_else(|| panic!("no spec resolved for {lang:?} + {fw:?}"));
            assert_eq!(spec.framework, fw);
            assert_eq!(spec.language, lang);
        }
    }
}

/// 纯语言路径：Rust/Python + None 可解析；Go/TS + None 不可（要求框架）。
#[test]
fn pure_language_paths_resolve_correctly() {
    assert!(resolve(Language::Rust, Framework::None).is_some());
    assert!(resolve(Language::Python, Framework::None).is_some());
    assert!(resolve(Language::Go, Framework::None).is_none());
    assert!(resolve(Language::TypeScript, Framework::None).is_none());
}

/// GoZero 必须标记为未实现。
#[test]
fn gozero_is_unimplemented() {
    let spec = resolve(Language::Go, Framework::GoZero).expect("gozero spec exists");
    assert_eq!(spec.kind, GenKind::Unimplemented);
}

/// proto/error-gen 仅 Rust(None) 与 Tauri 接受。
#[test]
fn proto_error_gen_only_for_rust_and_tauri() {
    assert!(
        resolve(Language::Rust, Framework::None)
            .unwrap()
            .accepts_proto_error_gen
    );
    assert!(
        resolve(Language::Rust, Framework::Tauri)
            .unwrap()
            .accepts_proto_error_gen
    );
    assert!(
        !resolve(Language::Go, Framework::Gin)
            .unwrap()
            .accepts_proto_error_gen
    );
    assert!(
        !resolve(Language::TypeScript, Framework::Vue3)
            .unwrap()
            .accepts_proto_error_gen
    );
}

/// 描述模板渲染保持与历史逐字一致。
#[test]
fn description_renders_project_name() {
    let spec = resolve(Language::Rust, Framework::Tauri).unwrap();
    assert_eq!(
        spec.description("myapp"),
        "A Tauri desktop application: myapp"
    );
}

/// FastAPI 解析为 Python + EmbeddedAsync，且不接受 proto/error-gen。
#[test]
fn fastapi_resolves_to_python_embedded_async() {
    let spec = resolve(Language::Python, Framework::FastApi).expect("fastapi spec exists");
    assert_eq!(spec.framework, Framework::FastApi);
    assert_eq!(spec.language, Language::Python);
    assert_eq!(spec.kind, GenKind::EmbeddedAsync);
    assert!(!spec.accepts_proto_error_gen);
    assert!(!spec.next_steps.is_empty());
}

/// McpServerPython 解析为 Python + EmbeddedAsync，且不接受 proto/error-gen。
#[test]
fn mcp_python_resolves_to_python_embedded_async() {
    let spec =
        resolve(Language::Python, Framework::McpServerPython).expect("mcp-python spec exists");
    assert_eq!(spec.framework, Framework::McpServerPython);
    assert_eq!(spec.language, Language::Python);
    assert_eq!(spec.kind, GenKind::EmbeddedAsync);
    assert!(!spec.accepts_proto_error_gen);
    assert!(!spec.next_steps.is_empty());
}

/// McpServer 解析为 Go + EmbeddedAsync，且不接受 proto/error-gen。
#[test]
fn mcp_server_resolves_to_go_embedded_async() {
    let spec = resolve(Language::Go, Framework::McpServer).expect("mcp-server spec exists");
    assert_eq!(spec.framework, Framework::McpServer);
    assert_eq!(spec.language, Language::Go);
    assert_eq!(spec.kind, GenKind::EmbeddedAsync);
    assert!(!spec.accepts_proto_error_gen);
    assert!(!spec.next_steps.is_empty());
}

/// `all_specs` 必须覆盖每一个 framework（含 None 的纯语言路径），且 GoZero 标记未实现。
#[test]
fn all_specs_covers_every_framework_and_flags_gozero() {
    let specs = all_specs();

    for fw in [
        Framework::Gin,
        Framework::GoZero,
        Framework::McpServer,
        Framework::FastApi,
        Framework::McpServerPython,
        Framework::Tauri,
        Framework::Vue3,
        Framework::React,
    ] {
        assert!(
            specs.iter().any(|s| s.framework == fw),
            "all_specs missing framework {fw:?}"
        );
    }

    let gozero = specs
        .iter()
        .find(|s| s.framework == Framework::GoZero)
        .expect("gozero present in all_specs");
    assert_eq!(gozero.kind, GenKind::Unimplemented);

    assert!(
        specs
            .iter()
            .any(|s| s.framework == Framework::None && s.language == Language::Rust)
    );
    assert!(
        specs
            .iter()
            .any(|s| s.framework == Framework::None && s.language == Language::Python)
    );
}
