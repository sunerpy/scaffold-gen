use anyhow::{Context, Result};
use std::path::Path;

use crate::generators::{
    core::Generator,
    framework::gin::{GinGenerator, GinParams},
    framework::react::{ReactGenerator, ReactParams},
    framework::tauri::{TauriGenerator, TauriParams},
    framework::vue3::{Vue3Generator, Vue3Params},
    language::go::{GoGenerator, GoParams},
    language::python::{PythonGenerator, PythonParams},
    language::rust::{RustGenerator, RustParams},
    project::{ProjectParams, ProjectScaffolder},
};
use crate::utils::env_checker::EnvironmentChecker;

/// 生成器编排器，负责协调三层架构的生成器
pub struct GeneratorOrchestrator {
    project_generator: ProjectScaffolder,
    go_generator: GoGenerator,
    python_generator: PythonGenerator,
    gin_generator: GinGenerator,
}

impl GeneratorOrchestrator {
    /// 创建新的生成器编排器
    pub fn new() -> Result<Self> {
        Ok(Self {
            project_generator: ProjectScaffolder::new()?,
            go_generator: GoGenerator::new()?,
            python_generator: PythonGenerator::new()?,
            gin_generator: GinGenerator::new()?,
        })
    }

    /// 生成完整的Gin项目
    pub fn generate_gin_project(
        &mut self,
        project_name: String,
        output_path: &Path,
        options: GinProjectOptions,
    ) -> Result<()> {
        println!("Starting Gin project generation: {project_name}");

        let module_name = options
            .module_name
            .unwrap_or_else(|| GoParams::infer_module_name(&project_name));
        let go_version = options.go_version.unwrap_or_else(|| "1.21".to_string());
        let license = options.license.unwrap_or_else(|| "MIT".to_string());
        let description = options
            .description
            .unwrap_or_else(|| format!("A Gin web application: {project_name}"));
        let author = options.author;
        let host = options.host.unwrap_or_else(|| "localhost".to_string());
        let port = options.port.unwrap_or(8080);
        let enable_git = options.enable_git.unwrap_or(true);
        let enable_precommit = options.enable_precommit.unwrap_or(true);
        let enable_swagger = options.enable_swagger.unwrap_or(true);
        let enable_cors = options.enable_cors.unwrap_or(true);
        let enable_jwt = options.enable_jwt.unwrap_or(false);
        let enable_redis = options.enable_redis.unwrap_or(false);
        let database_type = options.database_type;

        // 1. 框架级别生成 (Gin) - 首先生成应用结构
        let mut project_params = ProjectParams::new(project_name.clone())
            .with_description(description.clone())
            .with_license(license.clone());
        if let Some(author) = author.clone() {
            project_params = project_params.with_author(author);
        }

        let go_params = GoParams::new(module_name.clone()).with_version(go_version.clone());

        let mut gin_params = GinParams::from_project_name(project_name.clone())
            .with_project(project_params)
            .with_go(go_params)
            .with_server(host, port)
            .with_swagger(enable_swagger)
            .with_cors(enable_cors)
            .with_jwt(enable_jwt)
            .with_precommit(enable_precommit);

        if let Some(db_type) = database_type {
            gin_params = gin_params.with_database(db_type);
        }

        if enable_redis {
            gin_params = gin_params.with_redis(true);
        }

        self.gin_generator
            .generate(gin_params.clone(), output_path)
            .context("Failed to generate Gin framework files")?;

        // 2. 语言级别生成 (Go) - 然后执行 go mod init 和 go mod tidy
        let go_params = GoParams::new(module_name).with_version(go_version);

        self.go_generator
            .generate(go_params, output_path)
            .context("Failed to generate Go files")?;

        // 3. 项目级别生成 - 最后执行 git init 等项目级操作
        let mut project_params = ProjectParams::new(project_name.clone())
            .with_license(license)
            .with_git(enable_git)
            .with_precommit(enable_precommit)
            .with_description(description);

        if let Some(author) = author {
            project_params = project_params.with_author(author);
        }

        self.project_generator
            .generate(project_params, output_path)
            .context("Failed to generate project files")?;

        // 4. 执行后处理逻辑 - 在所有生成完成后执行 post_process
        self.gin_generator
            .post_process(&gin_params, output_path)
            .context("Failed to execute Gin post-processing")?;

        println!("Gin project generation completed successfully!");
        println!("Project created at: {}", output_path.display());

        Ok(())
    }

    /// 生成完整的Python项目
    pub async fn generate_python_project(
        &mut self,
        project_name: String,
        output_path: &Path,
        license: String,
        enable_precommit: bool,
    ) -> Result<()> {
        println!("Starting Python project generation: {project_name}");

        // 获取实际的 uv 版本和 Python 版本
        let env_checker = EnvironmentChecker::new();

        let uv_version = env_checker
            .get_uv_version()
            .await
            .unwrap_or_else(|_| "uv 0.9.5".to_string());

        // 从 "uv x.y.z" 格式中提取版本号
        let uv_version = uv_version
            .strip_prefix("uv ")
            .unwrap_or(&uv_version)
            .trim()
            .to_string();

        // 获取系统 Python 版本，如果获取失败则使用默认值
        let python_version = env_checker
            .get_python_version()
            .await
            .unwrap_or_else(|_| "3.12".to_string());

        // 1. 语言级别生成 (Python) - 使用 uv init 创建项目
        let python_params = PythonParams::new(project_name.clone())
            .with_version(python_version)
            .with_uv_version(uv_version)
            .with_precommit(enable_precommit);

        self.python_generator
            .generate(python_params, output_path)
            .context("Failed to generate Python files")?;

        // 2. 项目级别生成 - 生成 LICENSE、README 等
        let project_params = ProjectParams::new(project_name.clone())
            .with_license(license)
            .with_git(true)
            .with_precommit(enable_precommit)
            .with_description(format!("A Python project: {project_name}"));

        self.project_generator
            .generate(project_params, output_path)
            .context("Failed to generate project files")?;

        println!("Python project generation completed successfully!");
        println!("Project created at: {}", output_path.display());

        Ok(())
    }

    /// 生成完整的Rust项目
    pub async fn generate_rust_project(
        &mut self,
        project_name: String,
        output_path: &Path,
        license: String,
        enable_precommit: bool,
        enable_proto_gen: bool,
        enable_error_gen: bool,
    ) -> Result<()> {
        println!("Starting Rust project generation: {project_name}");

        // 获取实际的 Rust 版本
        let env_checker = EnvironmentChecker::new();
        let rust_version = env_checker
            .get_rust_version()
            .await
            .unwrap_or_else(|_| crate::constants::defaults::RUST_VERSION.to_string());

        // 1. 语言级别生成 (Rust) - 使用模板创建项目
        let rust_params = RustParams::new(project_name.clone())
            .with_rust_version(rust_version)
            .with_proto_gen(enable_proto_gen)
            .with_error_gen(enable_error_gen);

        // 设置 precommit
        let mut rust_params = rust_params;
        rust_params.base.enable_precommit = enable_precommit;

        RustGenerator::new()?
            .generate(rust_params, output_path)
            .context("Failed to generate Rust files")?;

        // 2. 项目级别生成 - 生成 LICENSE、README 等
        let project_params = ProjectParams::new(project_name.clone())
            .with_license(license)
            .with_git(true)
            .with_precommit(enable_precommit)
            .with_description(format!("A Rust project: {project_name}"));

        self.project_generator
            .generate(project_params, output_path)
            .context("Failed to generate project files")?;

        println!("Rust project generation completed successfully!");
        println!("Project created at: {}", output_path.display());

        Ok(())
    }

    /// 生成完整的Tauri项目
    pub async fn generate_tauri_project(
        &mut self,
        project_name: String,
        output_path: &Path,
        license: String,
        enable_precommit: bool,
        enable_proto_gen: bool,
        enable_error_gen: bool,
    ) -> Result<()> {
        println!("Starting Tauri project generation: {project_name}");

        // 1. 环境预检查
        println!("🔍 Checking environment prerequisites...");

        // 检查 pnpm
        if !TauriGenerator::check_pnpm()? {
            return Err(anyhow::anyhow!(
                "pnpm is not installed. Please install pnpm first:\n  npm install -g pnpm\n  or visit: https://pnpm.io/installation"
            ));
        }
        println!("  ✅ pnpm: Available");

        // 检查 create-tauri-app
        if !TauriGenerator::check_create_tauri_app()? {
            println!("  ⚠️ create-tauri-app not found, installing...");
            TauriGenerator::install_create_tauri_app()?;
        }
        println!("  ✅ create-tauri-app: Available");

        // 2. 删除已存在的目录（如果存在）
        if output_path.exists() {
            std::fs::remove_dir_all(output_path).context("Failed to remove existing directory")?;
        }

        // 3. 使用 create-tauri-app 创建项目
        TauriGenerator::create_tauri_project(&project_name, output_path)?;

        // 4. 安装前端依赖
        TauriGenerator::install_dependencies(output_path)?;

        // 5. 创建项目参数
        let project_params = ProjectParams::new(project_name.clone())
            .with_license(license.clone())
            .with_git(true)
            .with_precommit(enable_precommit)
            .with_description(format!("A Tauri desktop application: {project_name}"));

        // 6. 创建 Tauri 参数
        let tauri_params = TauriParams::from_project_name(project_name.clone())
            .with_project(project_params.clone())
            .with_precommit(enable_precommit)
            .with_proto_gen(enable_proto_gen)
            .with_error_gen(enable_error_gen);

        // 7. 覆盖模板文件 - 添加骨架屏、Tailwind CSS 等功能
        println!("📝 Applying enhanced templates...");
        TauriGenerator::new()?
            .generate(tauri_params, output_path)
            .context("Failed to apply Tauri templates")?;

        // 8. 重新安装依赖（因为 package.json 可能已更新）
        println!("📦 Reinstalling dependencies with updated package.json...");
        TauriGenerator::install_dependencies(output_path)?;

        // 9. 项目级别生成 - 生成 LICENSE 等
        self.project_generator
            .generate(project_params, output_path)
            .context("Failed to generate project files")?;

        println!("✅ Tauri project generation completed successfully!");
        println!("📁 Project created at: {}", output_path.display());
        println!("\n📋 Next steps:");
        println!("  cd {project_name}");
        println!("  cargo tauri dev    # Start development server");
        println!("  cargo tauri build  # Build for production");

        Ok(())
    }

    /// 生成完整的Vue3项目
    pub async fn generate_vue3_project(
        &mut self,
        project_name: String,
        output_path: &Path,
        license: String,
        enable_precommit: bool,
    ) -> Result<()> {
        println!("Starting Vue3 project generation: {project_name}");

        // 1. 环境预检查
        println!("🔍 Checking environment prerequisites...");

        // 检查 pnpm
        if !Vue3Generator::check_pnpm()? {
            return Err(anyhow::anyhow!(
                "pnpm is not installed. Please install pnpm first:\n  npm install -g pnpm\n  or visit: https://pnpm.io/installation"
            ));
        }
        println!("  ✅ pnpm: Available");

        // 2. 删除已存在的目录（如果存在）
        if output_path.exists() {
            std::fs::remove_dir_all(output_path).context("Failed to remove existing directory")?;
        }

        // 3. 使用 pnpm create vue 创建项目
        Vue3Generator::create_vue3_project(&project_name, output_path)?;

        // 4. 安装前端依赖
        Vue3Generator::install_dependencies(output_path)?;

        // 5. 安装 Tailwind CSS
        Vue3Generator::install_tailwind(output_path)?;

        // 6. 创建项目参数
        let project_params = ProjectParams::new(project_name.clone())
            .with_license(license.clone())
            .with_git(true)
            .with_precommit(enable_precommit)
            .with_description(format!("A Vue3 frontend application: {project_name}"));

        // 7. 创建 Vue3 参数
        let _vue3_params = Vue3Params::from_project_name(project_name.clone())
            .with_project(project_params.clone())
            .with_precommit(enable_precommit);

        // 8. 项目级别生成 - 生成 LICENSE 等
        self.project_generator
            .generate(project_params, output_path)
            .context("Failed to generate project files")?;

        println!("✅ Vue3 project generation completed successfully!");
        println!("📁 Project created at: {}", output_path.display());
        println!("\n📋 Next steps:");
        println!("  cd {project_name}");
        println!("  pnpm dev    # Start development server");
        println!("  pnpm build  # Build for production");

        Ok(())
    }

    /// 生成完整的React项目
    pub async fn generate_react_project(
        &mut self,
        project_name: String,
        output_path: &Path,
        license: String,
        enable_precommit: bool,
    ) -> Result<()> {
        println!("Starting React project generation: {project_name}");

        // 1. 环境预检查
        println!("🔍 Checking environment prerequisites...");

        // 检查 pnpm
        if !ReactGenerator::check_pnpm()? {
            return Err(anyhow::anyhow!(
                "pnpm is not installed. Please install pnpm first:\n  npm install -g pnpm\n  or visit: https://pnpm.io/installation"
            ));
        }
        println!("  ✅ pnpm: Available");

        // 2. 删除已存在的目录（如果存在）
        if output_path.exists() {
            std::fs::remove_dir_all(output_path).context("Failed to remove existing directory")?;
        }

        // 3. 使用 pnpm create vite 创建项目
        ReactGenerator::create_react_project(&project_name, output_path)?;

        // 4. 安装前端依赖
        ReactGenerator::install_dependencies(output_path)?;

        // 5. 安装 Tailwind CSS
        ReactGenerator::install_tailwind(output_path)?;

        // 6. 安装 React Router
        ReactGenerator::install_router(output_path)?;

        // 7. 安装状态管理库 (默认使用 zustand)
        ReactGenerator::install_state_management(output_path, "zustand")?;

        // 8. 创建项目参数
        let project_params = ProjectParams::new(project_name.clone())
            .with_license(license.clone())
            .with_git(true)
            .with_precommit(enable_precommit)
            .with_description(format!("A React frontend application: {project_name}"));

        // 9. 创建 React 参数
        let _react_params = ReactParams::from_project_name(project_name.clone())
            .with_project(project_params.clone())
            .with_precommit(enable_precommit);

        // 10. 项目级别生成 - 生成 LICENSE 等
        self.project_generator
            .generate(project_params, output_path)
            .context("Failed to generate project files")?;

        println!("✅ React project generation completed successfully!");
        println!("📁 Project created at: {}", output_path.display());
        println!("\n📋 Next steps:");
        println!("  cd {project_name}");
        println!("  pnpm dev    # Start development server");
        println!("  pnpm build  # Build for production");

        Ok(())
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

    /// 设置许可证
    pub fn with_license(mut self, license: String) -> Self {
        self.license = Some(license);
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
}
