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
use crate::generators::{GeneratorOrchestrator, GinProjectOptions};

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
        };

        self.generate_project(params).await?;

        tracing::info!("Project created successfully!");
        tracing::info!("Project path: {}", project_path.display());
        tracing::info!("Next steps:");
        tracing::info!("  cd {}", self.project_name);
        tracing::info!("  # Follow the README.md for further instructions");

        Ok(())
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
                gin_options,
            })
            .await
    }
}
