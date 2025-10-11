use anyhow::{Context, Result};
use std::path::Path;

use crate::generators::{
    core::Generator,
    framework::gin::{GinGenerator, GinParams},
    framework::go_zero::GoZeroGenerator,
    language::go::{GoGenerator, GoParams},
    project::{ProjectGenerator, ProjectParams},
};

/// 生成器编排器，负责协调三层架构的生成器
pub struct GeneratorOrchestrator {
    project_generator: ProjectGenerator,
    go_generator: GoGenerator,
    gin_generator: GinGenerator,
    #[allow(dead_code)]
    go_zero_generator: GoZeroGenerator,
}

impl GeneratorOrchestrator {
    /// 创建新的生成器编排器
    pub fn new() -> Result<Self> {
        Ok(Self {
            project_generator: ProjectGenerator::new()?,
            go_generator: GoGenerator::new()?,
            gin_generator: GinGenerator::new()?,
            go_zero_generator: GoZeroGenerator::new()?,
        })
    }

    /// 生成完整的Gin项目
    pub fn generate_gin_project(
        &mut self,
        project_name: String,
        output_path: &Path,
        options: GinProjectOptions,
    ) -> Result<()> {
        println!("🚀 Starting Gin project generation: {project_name}");

        // 1. 创建项目级别参数
        let project_params = ProjectParams::new(project_name.clone())
            .with_description(
                options
                    .description
                    .clone()
                    .unwrap_or_else(|| format!("A {project_name} project")),
            )
            .with_author(
                options
                    .author
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string()),
            )
            .with_license(options.license.clone().unwrap_or_else(|| "MIT".to_string()));

        // 2. 创建Go语言级别参数
        let go_params = GoParams::new(
            options
                .module_name
                .clone()
                .unwrap_or_else(|| GoParams::infer_module_name(&project_name)),
        )
        .with_version(
            options
                .go_version
                .clone()
                .unwrap_or_else(|| "1.21".to_string()),
        );

        // 3. 创建Gin框架级别参数
        let mut gin_params = GinParams::from_project_and_go(project_params, go_params)
            .with_server(
                options
                    .host
                    .clone()
                    .unwrap_or_else(|| "localhost".to_string()),
                options.port.unwrap_or(8080),
            )
            .with_swagger(options.enable_swagger.unwrap_or(true))
            .with_cors(options.enable_cors.unwrap_or(true))
            .with_jwt(options.enable_jwt.unwrap_or(false))
            .with_precommit(options.enable_precommit.unwrap_or(true));

        if let Some(db_type) = options.database_type {
            gin_params = gin_params.with_database(db_type);
        }

        if options.enable_redis.unwrap_or(false) {
            gin_params = gin_params.with_redis(true);
        }

        self.gin_generator
            .generate(gin_params.clone(), output_path)
            .context("Failed to generate Gin framework files")?;

        // 2. 语言级别生成 (Go) - 然后执行 go mod init 和 go mod tidy
        let module_name = options
            .module_name
            .unwrap_or_else(|| GoParams::infer_module_name(&project_name));

        let go_params = GoParams::new(module_name)
            .with_version(options.go_version.unwrap_or_else(|| "1.21".to_string()));

        self.go_generator
            .generate(go_params, output_path)
            .context("Failed to generate Go files")?;

        // 3. 项目级别生成 - 最后执行 git init 等项目级操作
        let mut project_params = ProjectParams::new(project_name.clone())
            .with_license(options.license.unwrap_or_else(|| "MIT".to_string()))
            .with_git(options.enable_git.unwrap_or(true))
            .with_description(
                options
                    .description
                    .unwrap_or_else(|| format!("A Gin web application: {project_name}")),
            );

        if let Some(author) = options.author {
            project_params = project_params.with_author(author);
        }

        self.project_generator
            .generate(project_params, output_path)
            .context("Failed to generate project files")?;

        // 4. 执行后处理逻辑 - 在所有生成完成后执行 post_process
        self.gin_generator
            .post_process(&gin_params, output_path)
            .context("Failed to execute Gin post-processing")?;

        println!("✅ Gin project generation completed successfully!");
        println!("📁 Project created at: {}", output_path.display());

        Ok(())
    }
}

impl Default for GeneratorOrchestrator {
    fn default() -> Self {
        Self::new().expect("Failed to create GeneratorOrchestrator")
    }
}

/// Gin项目生成选项
#[derive(Debug, Default)]
pub struct GinProjectOptions {
    // 项目级别选项
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub enable_git: Option<bool>,

    // 语言级别选项 (Go)
    pub go_version: Option<String>,
    pub module_name: Option<String>,

    // 框架级别选项 (Gin)
    pub host: Option<String>,
    pub port: Option<u16>,
    pub enable_swagger: Option<bool>,
    pub enable_cors: Option<bool>,
    pub enable_jwt: Option<bool>,
    pub enable_precommit: Option<bool>,
    pub enable_redis: Option<bool>,
    pub database_type: Option<String>,
}

impl GinProjectOptions {
    /// 创建新的选项
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置项目描述
    #[allow(dead_code)]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 设置作者
    #[allow(dead_code)]
    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }

    /// 设置许可证
    pub fn with_license(mut self, license: String) -> Self {
        self.license = Some(license);
        self
    }

    /// 设置Go版本
    #[allow(dead_code)]
    pub fn with_go_version(mut self, version: String) -> Self {
        self.go_version = Some(version);
        self
    }

    /// 设置模块名称
    #[allow(dead_code)]
    pub fn with_module_name(mut self, module_name: String) -> Self {
        self.module_name = Some(module_name);
        self
    }

    /// 设置服务器配置
    pub fn with_server(mut self, host: String, port: u16) -> Self {
        self.host = Some(host);
        self.port = Some(port);
        self
    }

    /// 启用Swagger
    pub fn with_swagger(mut self, enable: bool) -> Self {
        self.enable_swagger = Some(enable);
        self
    }

    /// 启用pre-commit
    pub fn with_precommit(mut self, enable: bool) -> Self {
        self.enable_precommit = Some(enable);
        self
    }

    /// 启用数据库
    #[allow(dead_code)]
    pub fn with_database(mut self, db_type: String) -> Self {
        self.database_type = Some(db_type);
        self
    }
}
