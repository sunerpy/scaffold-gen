use anyhow::{Context, Result};
use colored::*;
use inquire::{Confirm, Select, Text};
use std::path::PathBuf;

use crate::constants::{Framework, Language};
use crate::generators::{GeneratorOrchestrator, GinProjectOptions};
use crate::utils::env_checker::EnvironmentChecker;

/// Project generation parameters
struct ProjectParams {
    language: Language,
    framework: Framework,
    project_path: PathBuf,
    host: String,
    port: u16,
    enable_precommit: bool,
    license: String,
    enable_swagger: bool,
}

pub struct NewCommand {
    project_name: String,
    target_path: Option<String>,
    framework: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    grpc_port: Option<u16>,
    language: Option<String>,
    enable_precommit: Option<bool>,
    license: Option<String>,
    enable_swagger: Option<bool>,
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

    #[allow(dead_code)]
    pub fn with_swagger(mut self, enable_swagger: Option<bool>) -> Self {
        self.enable_swagger = enable_swagger;
        self
    }

    pub async fn execute(&self) -> Result<()> {
        println!("Welcome to Scaffold-Gen Project Generator!");

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
        };

        self.generate_project(params).await?;

        println!("Project created successfully!");
        println!("Project path: {}", project_path.display());
        println!("Next steps:");
        println!("  cd {}", self.project_name);
        println!("  # Follow the README.md for further instructions");

        Ok(())
    }

    async fn check_environment(&self, language: &Language) -> Result<()> {
        println!("Checking environment...");

        let env_checker = EnvironmentChecker::new();

        // 检查Git
        if !env_checker.check_git().await? {
            return Err(anyhow::anyhow!(
                "Git is not available. Please install Git first."
            ));
        }
        println!("  Git: Available");

        // 根据语言检查相应的环境
        match language {
            Language::Go => match env_checker.check_go().await {
                Ok(true) => println!("  Go: Available"),
                Ok(false) => {
                    return Err(anyhow::anyhow!(
                        "Go is not available. Please install Go first."
                    ));
                }
                Err(e) => return Err(anyhow::anyhow!("Go version check failed: {e}")),
            },
            Language::Python => match env_checker.check_uv().await {
                Ok(true) => println!("  uv: Available"),
                Ok(false) => {
                    return Err(anyhow::anyhow!(
                        "uv is not available. Please install uv first: https://docs.astral.sh/uv/"
                    ));
                }
                Err(e) => return Err(anyhow::anyhow!("uv check failed: {e}")),
            },
            Language::Rust => {
                // 检查 Cargo
                match env_checker.check_cargo().await {
                    Ok(true) => println!("  Cargo: Available"),
                    Ok(false) => {
                        return Err(anyhow::anyhow!(
                            "Cargo is not available. Please install Rust first: https://rustup.rs/"
                        ));
                    }
                    Err(e) => return Err(anyhow::anyhow!("Cargo check failed: {e}")),
                }

                // 如果选择了 Tauri 框架，还需要检查 pnpm
                if self.framework.as_ref().map(|f| f.to_lowercase()) == Some("tauri".to_string()) {
                    match env_checker.check_pnpm().await {
                        Ok(true) => println!("  pnpm: Available"),
                        Ok(false) => {
                            return Err(anyhow::anyhow!(
                                "pnpm is not available. Please install pnpm first:\n  npm install -g pnpm\n  or visit: https://pnpm.io/installation"
                            ));
                        }
                        Err(e) => return Err(anyhow::anyhow!("pnpm check failed: {e}")),
                    }
                }
            }
            Language::TypeScript => {
                // 检查 Node.js
                match env_checker.check_node().await {
                    Ok(true) => println!("  Node.js: Available"),
                    Ok(false) => {
                        return Err(anyhow::anyhow!(
                            "Node.js is not available. Please install Node.js first: https://nodejs.org/"
                        ));
                    }
                    Err(e) => return Err(anyhow::anyhow!("Node.js check failed: {e}")),
                }

                // 检查 pnpm
                match env_checker.check_pnpm().await {
                    Ok(true) => println!("  pnpm: Available"),
                    Ok(false) => {
                        return Err(anyhow::anyhow!(
                            "pnpm is not available. Please install pnpm first:\n  npm install -g pnpm\n  or visit: https://pnpm.io/installation"
                        ));
                    }
                    Err(e) => return Err(anyhow::anyhow!("pnpm check failed: {e}")),
                }
            }
        }

        Ok(())
    }

    fn select_language(&self) -> Result<Language> {
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
            println!("Programming language: {}", languages[0]);
            return Ok(languages[0]);
        }

        let selected = Select::new("Choose your programming language:", languages)
            .prompt()
            .context("Failed to select language")?;

        Ok(selected)
    }

    fn select_framework(&self, language: &Language) -> Result<Framework> {
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
            println!("Framework: {}", frameworks[0]);
            return Ok(frameworks[0]);
        }

        let selected = Select::new("Choose your framework:", frameworks)
            .prompt()
            .context("Failed to select framework")?;

        Ok(selected)
    }

    fn configure_network_settings(
        &self,
        framework: &Framework,
        language: &Language,
    ) -> Result<(String, u16, u16)> {
        // Rust、Python 和 TypeScript 语言不需要网络配置
        if matches!(
            language,
            Language::Rust | Language::Python | Language::TypeScript
        ) {
            return Ok(("0.0.0.0".to_string(), 8080, 9000));
        }

        println!("Configuring network settings...");

        let host = if let Some(ref h) = self.host {
            println!("Using provided host: {h}");
            h.clone()
        } else {
            println!("Prompting for host address...");
            Text::new("Host address:")
                .with_default("0.0.0.0")
                .prompt()
                .context("Failed to get host address")?
        };

        let port = if let Some(p) = self.port {
            println!("Using provided port: {p}");
            p
        } else {
            let default_port = match framework {
                Framework::None => 8080,
                Framework::Gin => 8080,
                Framework::GoZero => 8888,
                Framework::Tauri => 1420,
                Framework::Vue3 => 5173,
                Framework::React => 5173,
            };
            println!("Prompting for HTTP port...");
            Text::new("HTTP port:")
                .with_default(&default_port.to_string())
                .prompt()
                .context("Failed to get port")?
                .parse::<u16>()
                .context("Invalid port number")?
        };

        let grpc_port = if let Some(p) = self.grpc_port {
            println!("Using provided gRPC port: {p}");
            p
        } else if matches!(framework, Framework::GoZero) {
            println!("Prompting for gRPC port...");
            Text::new("gRPC port:")
                .with_default("9000")
                .prompt()
                .context("Failed to get gRPC port")?
                .parse::<u16>()
                .context("Invalid gRPC port number")?
        } else {
            println!("Using default gRPC port: 9000");
            9000 // 默认值，对于不需要gRPC的框架
        };

        Ok((host, port, grpc_port))
    }

    fn configure_precommit(&self) -> Result<bool> {
        println!("Configuring pre-commit settings...");

        if let Some(enable) = self.enable_precommit {
            println!("Using provided pre-commit setting: {enable}");
            Ok(enable)
        } else {
            println!("Prompting for pre-commit hooks...");
            Confirm::new("Enable pre-commit hooks?")
                .with_default(false)
                .prompt()
                .context("Failed to get pre-commit preference")
        }
    }

    fn configure_license(&self) -> Result<String> {
        println!("Configuring license...");

        if let Some(ref license) = self.license {
            println!("Using provided license: {license}");
            Ok(license.clone())
        } else {
            println!("Prompting for license selection...");
            let licenses = vec!["MIT", "Apache-2.0", "GPL-3.0", "BSD-3-Clause", "None"];
            Select::new("Select a license:", licenses)
                .prompt()
                .context("Failed to select license")
                .map(|s| s.to_string())
        }
    }

    async fn configure_swagger(&self, framework: &Framework, language: &Language) -> Result<bool> {
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
            println!(
                "{}",
                "⚠️  Swag command not found. Swagger documentation will be disabled.".yellow()
            );
            println!(
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

    fn determine_project_path(&self) -> Result<PathBuf> {
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

    async fn generate_project(&self, params: ProjectParams) -> Result<()> {
        println!("{}", "正在生成项目...".green());

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

        // 创建项目目录
        std::fs::create_dir_all(&params.project_path).with_context(|| {
            format!(
                "Failed to create project directory: {}",
                params.project_path.display()
            )
        })?;

        let mut orchestrator = GeneratorOrchestrator::new()?;

        // 根据框架类型生成项目
        match params.framework {
            Framework::Gin => {
                let options = GinProjectOptions::new()
                    .with_license(params.license.clone())
                    .with_server(params.host.clone(), params.port)
                    .with_swagger(params.enable_swagger)
                    .with_precommit(params.enable_precommit);

                orchestrator.generate_gin_project(
                    self.project_name.clone(),
                    &params.project_path,
                    options,
                )?;
            }
            Framework::GoZero => {
                // TODO: 实现 GoZero 项目生成
                return Err(anyhow::anyhow!("GoZero 项目生成尚未实现"));
            }
            Framework::Tauri => {
                orchestrator
                    .generate_tauri_project(
                        self.project_name.clone(),
                        &params.project_path,
                        params.license.clone(),
                        params.enable_precommit,
                    )
                    .await?;
            }
            Framework::Vue3 => {
                orchestrator
                    .generate_vue3_project(
                        self.project_name.clone(),
                        &params.project_path,
                        params.license.clone(),
                        params.enable_precommit,
                    )
                    .await?;
            }
            Framework::React => {
                orchestrator
                    .generate_react_project(
                        self.project_name.clone(),
                        &params.project_path,
                        params.license.clone(),
                        params.enable_precommit,
                    )
                    .await?;
            }
            Framework::None => {
                // 根据语言生成纯语言项目
                match params.language {
                    Language::Python => {
                        orchestrator
                            .generate_python_project(
                                self.project_name.clone(),
                                &params.project_path,
                                params.license.clone(),
                                params.enable_precommit,
                            )
                            .await?;
                    }
                    Language::Rust => {
                        orchestrator
                            .generate_rust_project(
                                self.project_name.clone(),
                                &params.project_path,
                                params.license.clone(),
                                params.enable_precommit,
                            )
                            .await?;
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "{} language requires a framework. Please choose one from: {}",
                            params.language,
                            valid_frameworks
                                .iter()
                                .map(|f| f.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}
