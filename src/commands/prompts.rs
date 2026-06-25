//! `NewCommand` 的交互式提示与环境检查 —— 从 `new.rs` 拆出的 `inquire` 驱动逻辑。
//!
//! 这里承载语言/框架选择、网络/许可证/swagger/pre-commit/Rust 工具配置、
//! 环境预检查与项目路径确定；命令骨架与生成调度仍留在 `new.rs`。

use anyhow::{Context, Result};
use inquire::{Confirm, Select, Text};
use std::path::PathBuf;

use super::new::NewCommand;
use crate::constants::{Framework, Language};
use crate::utils::env_checker::EnvironmentChecker;

impl NewCommand {
    pub(super) fn select_language(&self) -> Result<Language> {
        // 如果通过命令行参数指定了语言，直接使用
        if let Some(language_str) = &self.language {
            return match language_str.to_lowercase().as_str() {
                "go" => Ok(Language::Go),
                "python" => Ok(Language::Python),
                "rust" => Ok(Language::Rust),
                "typescript" | "ts" => Ok(Language::TypeScript),
                _ => Err(anyhow::anyhow!(
                    "Unsupported language: {language_str}. Supported languages: go, python, rust, typescript"
                )),
            };
        }

        let languages = vec![
            Language::Go,
            Language::Python,
            Language::Rust,
            Language::TypeScript,
        ];

        // 当只有一个选项时，直接返回该选项
        if languages.len() == 1 {
            tracing::info!("Programming language: {}", languages[0]);
            return Ok(languages[0]);
        }

        let selected = Select::new("Choose your programming language:", languages)
            .prompt()
            .context("Failed to select language")?;

        Ok(selected)
    }

    pub(super) fn select_framework(&self, language: &Language) -> Result<Framework> {
        // 获取该语言支持的框架列表
        let frameworks = Framework::frameworks_for_language(*language);

        // 如果没有可用框架（如 Python），返回 None
        if frameworks.is_empty() {
            return Ok(Framework::None);
        }

        // 如果通过命令行参数指定了框架，验证并使用
        if let Some(framework_str) = &self.framework {
            let framework = Framework::parse_from_str(framework_str).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unsupported framework: {framework_str}. Supported frameworks: gin, go-zero, tauri, vue3, react, none"
                )
            })?;

            // 验证框架是否适用于当前语言
            if !frameworks.contains(&framework) && framework != Framework::None {
                return Err(anyhow::anyhow!(
                    "Framework '{}' is not supported for {} language. Available frameworks: {}",
                    framework_str,
                    language,
                    frameworks
                        .iter()
                        .map(|f| f.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }

            return Ok(framework);
        }

        // 如果只有一个框架选项，直接返回
        if frameworks.len() == 1 {
            tracing::info!("Framework: {}", frameworks[0]);
            return Ok(frameworks[0]);
        }

        let selected = Select::new("Choose your framework:", frameworks)
            .prompt()
            .context("Failed to select framework")?;

        Ok(selected)
    }

    pub(super) fn configure_network_settings(
        &self,
        framework: &Framework,
        language: &Language,
    ) -> Result<(String, u16, u16)> {
        // Python FastAPI 是配置驱动的 API 服务，需要 host/port；其余 Python/Rust/TS
        // 纯项目不需要网络配置。
        let needs_network = matches!(language, Language::Go)
            || matches!(framework, Framework::FastApi | Framework::McpServer);
        if !needs_network {
            return Ok(("0.0.0.0".to_string(), 8080, 9000));
        }

        tracing::debug!("Configuring network settings...");

        let host = if let Some(ref h) = self.host {
            tracing::debug!("Using provided host: {h}");
            h.clone()
        } else {
            tracing::debug!("Prompting for host address...");
            Text::new("Host address:")
                .with_default("0.0.0.0")
                .prompt()
                .context("Failed to get host address")?
        };

        let port = if let Some(p) = self.port {
            tracing::debug!("Using provided port: {p}");
            p
        } else {
            let default_port = match framework {
                Framework::None => 8080,
                Framework::Gin => 8080,
                Framework::GoZero => 8888,
                Framework::McpServer => 8080,
                Framework::FastApi => 8000,
                Framework::Tauri => 1420,
                Framework::Vue3 => 5173,
                Framework::React => 5173,
            };
            tracing::debug!("Prompting for HTTP port...");
            Text::new("HTTP port:")
                .with_default(&default_port.to_string())
                .prompt()
                .context("Failed to get port")?
                .parse::<u16>()
                .context("Invalid port number")?
        };

        let grpc_port = if let Some(p) = self.grpc_port {
            tracing::debug!("Using provided gRPC port: {p}");
            p
        } else if matches!(framework, Framework::GoZero) {
            tracing::debug!("Prompting for gRPC port...");
            Text::new("gRPC port:")
                .with_default("9000")
                .prompt()
                .context("Failed to get gRPC port")?
                .parse::<u16>()
                .context("Invalid gRPC port number")?
        } else {
            tracing::debug!("Using default gRPC port: 9000");
            9000 // 默认值，对于不需要gRPC的框架
        };

        Ok((host, port, grpc_port))
    }

    pub(super) fn configure_precommit(&self) -> Result<bool> {
        tracing::debug!("Configuring pre-commit settings...");

        if let Some(enable) = self.enable_precommit {
            tracing::debug!("Using provided pre-commit setting: {enable}");
            Ok(enable)
        } else {
            tracing::debug!("Prompting for pre-commit hooks...");
            Confirm::new("Enable pre-commit hooks?")
                .with_default(false)
                .prompt()
                .context("Failed to get pre-commit preference")
        }
    }

    pub(super) fn configure_build(&self) -> Result<bool> {
        tracing::debug!("Configuring build tooling (Makefile + Dockerfile)...");

        if let Some(enable) = self.enable_build {
            tracing::debug!("Using provided build tooling setting: {enable}");
            Ok(enable)
        } else {
            tracing::debug!("Prompting for build tooling...");
            Confirm::new("生成配套 Makefile + Dockerfile (自动化构建/镜像)?")
                .with_default(false)
                .prompt()
                .context("Failed to get build tooling preference")
        }
    }

    pub(super) fn configure_license(&self) -> Result<String> {
        tracing::debug!("Configuring license...");

        if let Some(ref license) = self.license {
            tracing::debug!("Using provided license: {license}");
            Ok(license.clone())
        } else {
            tracing::debug!("Prompting for license selection...");
            let licenses = vec!["MIT", "Apache-2.0", "GPL-3.0", "BSD-3-Clause", "None"];
            Select::new("Select a license:", licenses)
                .prompt()
                .context("Failed to select license")
                .map(|s| s.to_string())
        }
    }

    pub(super) async fn configure_swagger(
        &self,
        framework: &Framework,
        language: &Language,
    ) -> Result<bool> {
        if let Some(enable_swagger) = self.enable_swagger {
            return Ok(enable_swagger);
        }

        // 只有 Go 语言的 Gin 框架支持 Swagger
        if !matches!(language, Language::Go) || !matches!(framework, Framework::Gin) {
            return Ok(false);
        }

        // 检查swag命令是否可用
        let env_checker = EnvironmentChecker::new();
        let swag_available = env_checker.check_swag().await.unwrap_or(false);

        if !swag_available {
            tracing::warn!("⚠️  Swag command not found. Swagger documentation will be disabled.");
            tracing::warn!(
                "   To enable Swagger, install swag: go install github.com/swaggo/swag/cmd/swag@latest"
            );
            return Ok(false);
        }

        let enable_swagger = Confirm::new("Enable Swagger documentation?")
            .with_default(true)
            .prompt()
            .context("Failed to configure Swagger")?;

        Ok(enable_swagger)
    }

    pub(super) fn configure_rust_tools(
        &self,
        language: &Language,
        framework: &Framework,
    ) -> Result<(bool, bool)> {
        if !matches!(language, Language::Rust) {
            return Ok((false, false));
        }

        // Tauri 和纯 Rust 项目都支持 proto-gen/error-gen
        if !matches!(framework, Framework::Tauri | Framework::None) {
            return Ok((false, false));
        }

        tracing::debug!("Configuring Rust code generation tools...");

        let enable_proto_gen = if let Some(enable) = self.enable_proto_gen {
            tracing::debug!("Using provided proto-gen setting: {enable}");
            enable
        } else {
            Confirm::new("Enable proto-gen? (Protobuf code generator)")
                .with_default(false)
                .prompt()
                .context("Failed to configure proto-gen")?
        };

        let enable_error_gen = if let Some(enable) = self.enable_error_gen {
            tracing::debug!("Using provided error-gen setting: {enable}");
            enable
        } else {
            Confirm::new("Enable error-gen? (Error type generator)")
                .with_default(false)
                .prompt()
                .context("Failed to configure error-gen")?
        };

        Ok((enable_proto_gen, enable_error_gen))
    }

    pub(super) fn determine_project_path(&self) -> Result<PathBuf> {
        let base_path = if let Some(path) = &self.target_path {
            PathBuf::from(path)
        } else {
            std::env::current_dir().context("Failed to get current directory")?
        };

        let project_path = base_path.join(&self.project_name);

        if project_path.exists() {
            return Err(anyhow::anyhow!(
                "Directory '{}' already exists",
                project_path.display()
            ));
        }

        Ok(project_path)
    }
}
