//! `scafgen new` 命令骨架 —— 命令结构、builder、`execute` 主流程与生成调度。
//!
//! 交互式提示与环境检查（select_/configure_/check_environment 等）拆到了
//! 同模块的 `prompts.rs`，本文件只保留命令外壳与到编排器的单一调度入口。

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::constants::Framework;
use crate::constants::Language;
use crate::generators::orchestrator::GenerationRequest;
use crate::generators::registry;
use crate::generators::{AuthMode, GeneratorOrchestrator, GinProjectOptions, McpBackend};

/// Project generation parameters
pub(super) struct ProjectParams {
    pub(super) language: Language,
    pub(super) framework: Framework,
    pub(super) project_path: PathBuf,
    pub(super) host: String,
    pub(super) port: u16,
    pub(super) enable_precommit: bool,
    pub(super) license: String,
    pub(super) enable_swagger: bool,
    pub(super) enable_proto_gen: bool,
    pub(super) enable_error_gen: bool,
    pub(super) enable_build: bool,
    pub(super) mcp_backend: McpBackend,
    pub(super) auth_mode: AuthMode,
}

pub struct NewCommand {
    pub(super) project_name: String,
    pub(super) target_path: Option<String>,
    pub(super) framework: Option<String>,
    pub(super) host: Option<String>,
    pub(super) port: Option<u16>,
    pub(super) grpc_port: Option<u16>,
    pub(super) language: Option<String>,
    pub(super) enable_precommit: Option<bool>,
    pub(super) license: Option<String>,
    pub(super) enable_swagger: Option<bool>,
    pub(super) enable_proto_gen: Option<bool>,
    pub(super) enable_error_gen: Option<bool>,
    pub(super) enable_build: Option<bool>,
    pub(super) mcp_backend: Option<String>,
    pub(super) auth_mode: Option<String>,
}

impl NewCommand {
    pub fn new(project_name: String, target_path: Option<String>) -> Self {
        Self {
            project_name,
            target_path,
            framework: None,
            host: None,
            port: None,
            grpc_port: None,
            language: None,
            enable_precommit: None,
            license: None,
            enable_swagger: None,
            enable_proto_gen: None,
            enable_error_gen: None,
            enable_build: None,
            mcp_backend: None,
            auth_mode: None,
        }
    }

    pub fn with_framework(mut self, framework: Option<String>) -> Self {
        self.framework = framework;
        self
    }

    pub fn with_host(mut self, host: Option<String>) -> Self {
        self.host = host;
        self
    }

    pub fn with_port(mut self, port: Option<u16>) -> Self {
        self.port = port;
        self
    }

    pub fn with_grpc_port(mut self, grpc_port: Option<u16>) -> Self {
        self.grpc_port = grpc_port;
        self
    }

    pub fn with_language(mut self, language: Option<String>) -> Self {
        self.language = language;
        self
    }

    pub fn with_precommit(mut self, enable_precommit: Option<bool>) -> Self {
        self.enable_precommit = enable_precommit;
        self
    }

    pub fn with_license(mut self, license: Option<String>) -> Self {
        self.license = license;
        self
    }

    pub fn with_swagger(mut self, enable_swagger: Option<bool>) -> Self {
        self.enable_swagger = enable_swagger;
        self
    }

    pub fn with_proto_gen(mut self, enable_proto_gen: Option<bool>) -> Self {
        self.enable_proto_gen = enable_proto_gen;
        self
    }

    pub fn with_error_gen(mut self, enable_error_gen: Option<bool>) -> Self {
        self.enable_error_gen = enable_error_gen;
        self
    }

    pub fn with_build(mut self, enable_build: Option<bool>) -> Self {
        self.enable_build = enable_build;
        self
    }

    pub fn with_mcp_backend(mut self, mcp_backend: Option<String>) -> Self {
        self.mcp_backend = mcp_backend;
        self
    }

    pub fn with_auth_mode(mut self, auth_mode: Option<String>) -> Self {
        self.auth_mode = auth_mode;
        self
    }

    fn resolve_mcp_backend(&self, framework: &Framework) -> Result<McpBackend> {
        if let Some(value) = &self.mcp_backend {
            return McpBackend::parse_from_str(value).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unsupported mcp backend: {value}. Supported backends: fastmcp, official"
                )
            });
        }

        if *framework == Framework::McpServerPython {
            return self.select_mcp_backend();
        }

        Ok(McpBackend::Fastmcp)
    }

    fn resolve_auth_mode(&self, framework: &Framework) -> Result<AuthMode> {
        if let Some(value) = &self.auth_mode {
            return AuthMode::parse_from_str(value).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unsupported auth mode: {value}. Supported modes: none, jwt, azure-ad"
                )
            });
        }

        if *framework == Framework::McpServerPython {
            return self.select_auth_mode();
        }

        Ok(AuthMode::None)
    }

    pub async fn execute(&self) -> Result<()> {
        tracing::info!("Welcome to Scaffold-Gen Project Generator!");

        // 交互式选择
        let language = self.select_language()?;

        // 环境检查
        self.check_environment(&language).await?;

        let framework = self.select_framework(&language)?;

        // 配置选项
        let (host, port, _grpc_port) = self.configure_network_settings(&framework, &language)?;
        let enable_precommit = self.configure_precommit()?;
        let license = self.configure_license()?;
        let enable_swagger = self.configure_swagger(&framework, &language).await?;

        // 配置 Rust 工具选项 (proto-gen / error-gen)
        let (enable_proto_gen, enable_error_gen) =
            self.configure_rust_tools(&language, &framework)?;

        // 配置构建工具链 (Makefile + Dockerfile)
        let enable_build = self.configure_build()?;

        // 解析 mcp-python 后端 (flag / 交互 / 默认 fastmcp)
        let mcp_backend = self.resolve_mcp_backend(&framework)?;

        // 解析 mcp-python 鉴权模式 (flag / 交互 / 默认 none)
        let auth_mode = self.resolve_auth_mode(&framework)?;

        // 确定项目路径
        let project_path = self.determine_project_path()?;

        // 生成项目
        let params = ProjectParams {
            language,
            framework,
            project_path: project_path.clone(),
            host,
            port,
            enable_precommit,
            license,
            enable_swagger,
            enable_proto_gen,
            enable_error_gen,
            enable_build,
            mcp_backend,
            auth_mode,
        };

        let equivalent_command = self.equivalent_command(&params);

        self.generate_project(params).await?;

        tracing::info!("Project created successfully!");
        tracing::info!("Project path: {}", project_path.display());
        tracing::info!("Next steps:");
        tracing::info!("  cd {}", self.project_name);
        tracing::info!("  # Follow the README.md for further instructions");

        tracing::info!("");
        tracing::info!("💡 Equivalent non-interactive command (等价的非交互命令):");
        tracing::info!("  {equivalent_command}");

        Ok(())
    }

    /// 根据交互式流程解析出的 `ProjectParams` 构造等价的非交互命令行。
    ///
    /// 仅输出对所选语言/框架有意义的 flag（镜像 `configure_*` 的同款条件）：
    /// - `--framework` 在 `None`（纯语言路径）时省略；
    /// - `--host`/`--port` 仅在需要网络的组合（Go，或 FastApi/McpServer）时输出；
    /// - `--swagger` 仅 Gin；`--proto-gen`/`--error-gen` 仅 Rust(Tauri|None)。
    ///
    /// flag 顺序稳定（便于测试与阅读），输出可直接重新运行。
    pub(super) fn equivalent_command(&self, params: &ProjectParams) -> String {
        let mut parts: Vec<String> = vec![
            "scafgen".to_string(),
            "new".to_string(),
            quote_if_needed(&self.project_name),
        ];

        // -p 是「父目录」（project_path = base.join(name)），仅在非默认父目录时输出
        if let Some(parent) = params.project_path.parent() {
            let parent_str = parent.display().to_string();
            if !parent_str.is_empty() && parent_str != "." {
                parts.push("-p".to_string());
                parts.push(quote_if_needed(&parent_str));
            }
        }

        parts.push("--language".to_string());
        parts.push(params.language.build_dir().to_string());

        if params.framework != Framework::None {
            parts.push("--framework".to_string());
            // parser 接受小写；Gin 的 as_str() 是 "Gin"，统一小写后仍可解析
            parts.push(params.framework.as_str().to_lowercase());
        }

        // host/port 仅对需要网络的组合有意义（镜像 configure_network_settings）
        let needs_network = matches!(params.language, Language::Go)
            || matches!(
                params.framework,
                Framework::FastApi | Framework::McpServer | Framework::McpServerPython
            );
        if needs_network {
            parts.push("--host".to_string());
            parts.push(quote_if_needed(&params.host));
            parts.push("--port".to_string());
            parts.push(params.port.to_string());
        }

        parts.push("--precommit".to_string());
        parts.push(params.enable_precommit.to_string());
        parts.push("--license".to_string());
        parts.push(quote_if_needed(&params.license));

        if params.framework == Framework::Gin {
            parts.push("--swagger".to_string());
            parts.push(params.enable_swagger.to_string());
        }

        // proto-gen / error-gen 仅 Rust 的 Tauri/None 路径有意义（镜像 configure_rust_tools）
        if matches!(params.language, Language::Rust)
            && matches!(params.framework, Framework::Tauri | Framework::None)
        {
            parts.push("--proto-gen".to_string());
            parts.push(params.enable_proto_gen.to_string());
            parts.push("--error-gen".to_string());
            parts.push(params.enable_error_gen.to_string());
        }

        parts.push("--with-build".to_string());
        parts.push(params.enable_build.to_string());

        if params.framework == Framework::McpServerPython {
            parts.push("--mcp-backend".to_string());
            parts.push(params.mcp_backend.as_str().to_string());
        }

        if params.framework == Framework::McpServerPython && params.auth_mode != AuthMode::None {
            parts.push("--auth".to_string());
            parts.push(params.auth_mode.as_str().to_string());
        }

        parts.join(" ")
    }

    async fn generate_project(&self, params: ProjectParams) -> Result<()> {
        tracing::info!("正在生成项目...");

        // 验证语言和框架组合是否有效
        let valid_frameworks = Framework::frameworks_for_language(params.language);
        if !valid_frameworks.is_empty()
            && !valid_frameworks.contains(&params.framework)
            && params.framework != Framework::None
        {
            return Err(anyhow::anyhow!(
                "Framework '{}' is not supported for {} language. Available frameworks: {}",
                params.framework.as_str(),
                params.language,
                valid_frameworks
                    .iter()
                    .map(|f| f.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        // 单一调度点：把 (语言, 框架) 解析为唯一的生成规格
        let spec = registry::resolve(params.language, params.framework).ok_or_else(|| {
            anyhow::anyhow!(
                "{} language requires a framework. Please choose one from: {}",
                params.language,
                valid_frameworks
                    .iter()
                    .map(|f| f.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;

        // 创建项目目录
        std::fs::create_dir_all(&params.project_path).with_context(|| {
            format!(
                "Failed to create project directory: {}",
                params.project_path.display()
            )
        })?;

        let gin_options = GinProjectOptions::new()
            .with_license(params.license.clone())
            .with_server(params.host.clone(), params.port)
            .with_swagger(params.enable_swagger)
            .with_precommit(params.enable_precommit);

        let mut orchestrator = GeneratorOrchestrator::new()?;
        orchestrator
            .generate(GenerationRequest {
                spec,
                project_name: self.project_name.clone(),
                output_path: &params.project_path,
                license: params.license.clone(),
                enable_precommit: params.enable_precommit,
                enable_proto_gen: params.enable_proto_gen,
                enable_error_gen: params.enable_error_gen,
                enable_build: params.enable_build,
                gin_options,
                mcp_backend: params.mcp_backend,
                auth_mode: params.auth_mode,
            })
            .await
    }
}

/// 仅在值含空格或 shell 元字符时加单引号，否则原样返回，保证输出可直接重跑。
fn quote_if_needed(value: &str) -> String {
    let needs_quote = value.is_empty()
        || value
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '"' | '\'' | '$' | '`' | '\\'));
    if needs_quote {
        format!("'{}'", value.replace('\'', r"'\''"))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
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
    fn rust_none_omits_framework_network_and_includes_rust_tool_flags() {
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
            "scafgen new mylib -p /tmp/work --language rust --precommit false \
             --license MIT --proto-gen false --error-gen false --with-build false"
        );
        assert!(!cmd.contains("--framework"));
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
            "scafgen new plain --language python --precommit true \
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
             --with-build false --mcp-backend official"
        );
        assert!(cmd.contains("--mcp-backend official"));
        assert!(!cmd.contains("--auth"));
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
}
