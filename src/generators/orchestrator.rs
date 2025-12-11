use anyhow::{Context, Result};
use std::path::Path;

use crate::generators::{
    core::Generator,
    framework::gin::{GinGenerator, GinParams},
    framework::go_zero::GoZeroGenerator,
    framework::react::{ReactGenerator, ReactParams},
    framework::tauri::{TauriGenerator, TauriParams},
    framework::vue3::{Vue3Generator, Vue3Params},
    language::go::{GoGenerator, GoParams},
    language::python::{PythonGenerator, PythonParams},
    language::rust::{RustGenerator, RustParams},
    project::{ProjectGenerator, ProjectParams},
};
use crate::utils::env_checker::EnvironmentChecker;

/// 生成器编排器，负责协调三层架构的生成器
pub struct GeneratorOrchestrator {
    project_generator: ProjectGenerator,
    go_generator: GoGenerator,
    python_generator: PythonGenerator,
    #[allow(dead_code)]
    rust_generator: RustGenerator,
    gin_generator: GinGenerator,
    #[allow(dead_code)]
    go_zero_generator: GoZeroGenerator,
    #[allow(dead_code)]
    tauri_generator: TauriGenerator,
    #[allow(dead_code)]
    vue3_generator: Vue3Generator,
    #[allow(dead_code)]
    react_generator: ReactGenerator,
}

impl GeneratorOrchestrator {
    /// 创建新的生成器编排器
    pub fn new() -> Result<Self> {
        Ok(Self {
            project_generator: ProjectGenerator::new()?,
            go_generator: GoGenerator::new()?,
            python_generator: PythonGenerator::new()?,
            rust_generator: RustGenerator::new()?,
            gin_generator: GinGenerator::new()?,
            go_zero_generator: GoZeroGenerator::new()?,
            tauri_generator: TauriGenerator::new()?,
            vue3_generator: Vue3Generator::new()?,
            react_generator: ReactGenerator::new()?,
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
        let mut gin_params = GinParams::from_project_name(project_name.clone())
            .with_project(project_params)
            .with_go(go_params)
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
            .with_precommit(options.enable_precommit.unwrap_or(true))
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
    #[allow(dead_code)]
    pub async fn generate_rust_project(
        &mut self,
        project_name: String,
        output_path: &Path,
        license: String,
        enable_precommit: bool,
    ) -> Result<()> {
        println!("Starting Rust project generation: {project_name}");

        // 获取实际的 Rust 版本
        let env_checker = EnvironmentChecker::new();
        let rust_version = env_checker
            .get_rust_version()
            .await
            .unwrap_or_else(|_| crate::constants::defaults::RUST_VERSION.to_string());

        // 1. 语言级别生成 (Rust) - 使用 cargo init 创建项目
        let rust_params = RustParams::new(project_name.clone()).with_rust_version(rust_version);

        self.rust_generator
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
            .with_precommit(enable_precommit);

        // 7. 覆盖模板文件 - 添加骨架屏、Tailwind CSS 等功能
        println!("📝 Applying enhanced templates...");
        self.tauri_generator
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
