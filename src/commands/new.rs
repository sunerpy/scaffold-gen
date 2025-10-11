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
        let (host, port, _grpc_port) = self.configure_network_settings(&framework)?;
        let enable_precommit = self.configure_precommit()?;
        let license = self.configure_license()?;
        let enable_swagger = self.configure_swagger(&framework).await?;

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
            Language::Python => {
                // TODO: 实现 Python 环境检查
                println!("  Python: Environment check not implemented yet");
            }
            Language::Rust => {
                // TODO: 实现 Rust 环境检查
                println!("  Rust: Environment check not implemented yet");
            }
        }

        Ok(())
    }

    fn select_language(&self) -> Result<Language> {
        // 如果通过命令行参数指定了语言，直接使用
        if let Some(language_str) = &self.language {
            return match language_str.to_lowercase().as_str() {
                "go" => Ok(Language::Go),
                _ => Err(anyhow::anyhow!(
                    "Unsupported language: {language_str}. Supported languages: go"
                )),
            };
        }

        let languages = vec![Language::Go];

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

    fn select_framework(&self, _language: &Language) -> Result<Framework> {
        // 如果通过命令行参数指定了框架，直接使用
        if let Some(framework_str) = &self.framework {
            return Framework::parse_from_str(framework_str).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unsupported framework: {framework_str}. Supported frameworks: gin, go-zero"
                )
            });
        }

        // 否则使用交互式选择
        let frameworks = vec![Framework::Gin, Framework::GoZero];

        let selected = Select::new("Choose your framework:", frameworks)
            .prompt()
            .context("Failed to select framework")?;

        Ok(selected)
    }

    fn configure_network_settings(&self, framework: &Framework) -> Result<(String, u16, u16)> {
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
                Framework::Gin => 8080,
                Framework::GoZero => 8888,
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

    async fn configure_swagger(&self, framework: &Framework) -> Result<bool> {
        if let Some(enable_swagger) = self.enable_swagger {
            return Ok(enable_swagger);
        }

        // 只有Gin框架支持Swagger
        if !matches!(framework, Framework::Gin) {
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

        // 创建项目目录
        std::fs::create_dir_all(&params.project_path).with_context(|| {
            format!(
                "Failed to create project directory: {}",
                params.project_path.display()
            )
        })?;

        let mut orchestrator = GeneratorOrchestrator::new()?;

        match (&params.language, &params.framework) {
            (Language::Go, Framework::Gin) => {
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
            (Language::Go, Framework::GoZero) => {
                // TODO: 实现 GoZero 项目生成
                return Err(anyhow::anyhow!("GoZero 项目生成尚未实现"));
            }
            (Language::Python, _) => {
                // TODO: 实现 Python 项目生成
                return Err(anyhow::anyhow!("Python 项目生成尚未实现"));
            }
            (Language::Rust, _) => {
                // TODO: 实现 Rust 项目生成
                return Err(anyhow::anyhow!("Rust 项目生成尚未实现"));
            }
        }

        Ok(())
    }
}
