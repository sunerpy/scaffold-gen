use crate::constants::{Framework, Language};

/// 框架生成策略 - 描述一个框架如何被生成（数据驱动调度的核心）
///
/// 这是替代旧版 `generate_project` 中 78 行手写 `match` 的关键：
/// 框架之间的差异以 *数据* 表达，而不是控制流分支。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenKind {
    /// 同步生成（Gin）：通过 `GinProjectOptions` 读取 host/port/swagger/precommit/license。
    GinSync,
    /// 异步、纯语言的内嵌模板生成（Python / 纯 Rust）。
    EmbeddedAsync,
    /// 异步、外部脚手架生成（Tauri / Vue3 / React，会 shell 到 pnpm/create-tauri-app）。
    ExternalAsync,
    /// 尚未实现（GoZero）。
    Unimplemented,
}

/// 单个框架的生成元数据 —— 唯一真相来源。
///
/// 新增一个内嵌模板框架 = 在 `REGISTRY` 中加一条记录 + 一个模板目录，
/// 无需在 `new.rs` 中新增 `match` 分支。
#[derive(Debug, Clone, Copy)]
pub struct FrameworkSpec {
    /// 该规格对应的框架枚举值。
    pub framework: Framework,
    /// 该框架所属的语言。
    pub language: Language,
    /// 生成策略。
    pub kind: GenKind,
    /// 项目描述模板，`{name}` 会被替换为项目名（保持与历史输出逐字一致）。
    pub description_template: &'static str,
    /// 是否接受 proto-gen / error-gen 特性开关（仅 Rust / Tauri 为 true）。
    pub accepts_proto_error_gen: bool,
    /// 生成完成后打印的 "Next steps" 提示行（不含框架自身在生成内部打印的提示）。
    pub next_steps: &'static [&'static str],
    /// 该框架是否自带 Makefile（`--with-build` 时跳过通用 Makefile 渲染）。
    pub has_own_makefile: bool,
}

impl FrameworkSpec {
    /// 渲染项目描述，将 `{name}` 替换为项目名。
    pub fn description(&self, project_name: &str) -> String {
        self.description_template.replace("{name}", project_name)
    }
}

/// 框架规格注册表 —— 每个 `Framework` 变体（除纯语言路径外按语言归属）对应一条记录。
///
/// `Framework::None` 不在此处，它会按 *语言* 解析（见 [`resolve`]）。
const REGISTRY: &[FrameworkSpec] = &[
    FrameworkSpec {
        framework: Framework::Gin,
        language: Language::Go,
        kind: GenKind::GinSync,
        description_template: "A Gin web application: {name}",
        accepts_proto_error_gen: false,
        next_steps: &[],
        has_own_makefile: false,
    },
    FrameworkSpec {
        framework: Framework::GoZero,
        language: Language::Go,
        kind: GenKind::Unimplemented,
        description_template: "A go-zero microservice: {name}",
        accepts_proto_error_gen: false,
        next_steps: &[],
        has_own_makefile: false,
    },
    FrameworkSpec {
        framework: Framework::McpServer,
        language: Language::Go,
        kind: GenKind::EmbeddedAsync,
        description_template: "A Go MCP server (Gin + go-sdk): {name}",
        accepts_proto_error_gen: false,
        next_steps: &[
            "  make generate                 # buf generate: proto + JSON Schema",
            "  make run                      # Start the MCP server (reads config.toml)",
        ],
        has_own_makefile: true,
    },
    FrameworkSpec {
        framework: Framework::FastApi,
        language: Language::Python,
        kind: GenKind::EmbeddedAsync,
        description_template: "A FastAPI service: {name}",
        accepts_proto_error_gen: false,
        next_steps: &[
            "  uv sync                       # Install dependencies",
            "  uv run python main.py         # Start the API (reads config.toml / .env)",
        ],
        has_own_makefile: true,
    },
    FrameworkSpec {
        framework: Framework::McpServerPython,
        language: Language::Python,
        kind: GenKind::EmbeddedAsync,
        description_template: "A Python MCP server (FastMCP, streamable-HTTP + SSE): {name}",
        accepts_proto_error_gen: false,
        next_steps: &[
            "  uv sync                       # Install dependencies",
            "  make test                     # Run the tool tests (pytest, in-memory client)",
            "  uv run python main.py         # Start the MCP server (reads config.toml / .env)",
        ],
        has_own_makefile: true,
    },
    FrameworkSpec {
        framework: Framework::Tauri,
        language: Language::Rust,
        kind: GenKind::ExternalAsync,
        description_template: "A Tauri desktop application: {name}",
        accepts_proto_error_gen: true,
        next_steps: &[
            "  cargo tauri dev    # Start development server",
            "  cargo tauri build  # Build for production",
        ],
        has_own_makefile: false,
    },
    FrameworkSpec {
        framework: Framework::Vue3,
        language: Language::TypeScript,
        kind: GenKind::EmbeddedAsync,
        description_template: "A Vue3 frontend application: {name}",
        accepts_proto_error_gen: false,
        next_steps: &[
            "  pnpm dev    # Start development server",
            "  pnpm build  # Build for production",
        ],
        has_own_makefile: false,
    },
    FrameworkSpec {
        framework: Framework::React,
        language: Language::TypeScript,
        kind: GenKind::ExternalAsync,
        description_template: "A React frontend application: {name}",
        accepts_proto_error_gen: false,
        next_steps: &[
            "  pnpm dev    # Start development server",
            "  pnpm build  # Build for production",
        ],
        has_own_makefile: false,
    },
];

/// 纯语言（`Framework::None`）路径的生成规格 —— 按语言解析。
const fn pure_language_spec(language: Language) -> Option<FrameworkSpec> {
    match language {
        Language::Python => Some(FrameworkSpec {
            framework: Framework::None,
            language: Language::Python,
            kind: GenKind::EmbeddedAsync,
            description_template: "A Python project: {name}",
            accepts_proto_error_gen: false,
            next_steps: &[],
            has_own_makefile: false,
        }),
        Language::Rust => Some(FrameworkSpec {
            framework: Framework::None,
            language: Language::Rust,
            kind: GenKind::EmbeddedAsync,
            description_template: "A Rust project: {name}",
            accepts_proto_error_gen: true,
            next_steps: &[],
            has_own_makefile: false,
        }),
        // Go / TypeScript 没有 "纯语言" 路径 —— 它们要求一个框架。
        Language::Go | Language::TypeScript => None,
    }
}

/// 枚举所有 (语言, 框架) 组合对应的生成规格 —— 供 `scafgen list` 自省使用。
///
/// 数据来源完全由 `Framework::frameworks_for_language` + [`resolve`] 驱动，
/// 因此与调度逻辑共享唯一真相来源：注册表新增/删除一行，`list` 输出自动跟随。
/// 遍历顺序按 Go → Python → Rust → TypeScript，组内按 `frameworks_for_language`
/// 的声明顺序，保证输出稳定可读。
pub fn all_specs() -> Vec<FrameworkSpec> {
    let langs = [
        Language::Go,
        Language::Python,
        Language::Rust,
        Language::TypeScript,
    ];
    let mut specs = Vec::new();
    for lang in langs {
        for fw in Framework::frameworks_for_language(lang) {
            if let Some(spec) = resolve(lang, fw) {
                specs.push(spec);
            }
        }
    }
    specs
}

/// 单一调度入口：把 `(language, framework)` 解析为唯一的生成规格。
///
/// 返回 `None` 表示该组合没有对应的纯语言路径（Go/TS + None），
/// 调用方据此产生 "language requires a framework" 错误。
pub fn resolve(language: Language, framework: Framework) -> Option<FrameworkSpec> {
    if framework == Framework::None {
        return pure_language_spec(language);
    }

    REGISTRY
        .iter()
        .copied()
        .find(|spec| spec.framework == framework)
}

#[cfg(test)]
mod tests {
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
}
