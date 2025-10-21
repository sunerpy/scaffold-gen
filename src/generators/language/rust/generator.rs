use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::constants::Language;
use crate::generators::core::{
    Generator, InheritableParams, LanguageGenerator as LanguageGeneratorTrait, Parameters,
    TemplateProcessor,
};
use crate::generators::language::rust::parameters::RustParams;

/// Rust 语言生成器
pub struct RustGenerator {}

impl RustGenerator {
    /// 创建新的 Rust 生成器
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// 使用 cargo init 初始化项目
    fn init_cargo_project(&self, params: &RustParams, output_path: &Path) -> Result<()> {
        println!("Initializing Rust project with cargo...");

        let project_name = &params.base_params().project_name;

        // 使用 cargo init 创建项目
        let status = Command::new("cargo")
            .arg("init")
            .arg("--name")
            .arg(project_name)
            .arg(output_path)
            .status()
            .context("Failed to execute cargo init")?;

        if !status.success() {
            return Err(anyhow::anyhow!("cargo init failed"));
        }

        println!("Rust project initialized with cargo");
        Ok(())
    }

    /// 添加必要的依赖
    fn add_dependencies(&self, output_path: &Path) -> Result<()> {
        println!("Adding Rust dependencies...");

        // 不指定版本,让 cargo 自动选择最新的兼容版本
        let dependencies = vec![
            "tracing",
            "tracing-appender",
            "anyhow",
            "serde_json",
            "config",
        ];

        for dep in dependencies {
            let status = Command::new("cargo")
                .arg("add")
                .arg(dep)
                .current_dir(output_path)
                .status()
                .context(format!("Failed to add dependency: {}", dep))?;

            if !status.success() {
                println!("Warning: Failed to add dependency {}", dep);
            }
        }

        // 添加带特性的依赖,不指定版本
        let status = Command::new("cargo")
            .arg("add")
            .arg("tokio")
            .arg("--features")
            .arg("full")
            .current_dir(output_path)
            .status()
            .context("Failed to add tokio with features")?;

        if !status.success() {
            println!("Warning: Failed to add tokio with features");
        }

        let status = Command::new("cargo")
            .arg("add")
            .arg("serde")
            .arg("--features")
            .arg("derive")
            .current_dir(output_path)
            .status()
            .context("Failed to add serde with features")?;

        if !status.success() {
            println!("Warning: Failed to add serde with features");
        }

        let status = Command::new("cargo")
            .arg("add")
            .arg("tracing-subscriber")
            .arg("--features")
            .arg("json,env-filter,chrono")
            .current_dir(output_path)
            .status()
            .context("Failed to add tracing-subscriber with features")?;

        if !status.success() {
            println!("Warning: Failed to add tracing-subscriber with features");
        }

        println!("Dependencies added successfully");
        Ok(())
    }

    /// 构建项目以验证依赖
    fn build_project(&self, output_path: &Path) -> Result<()> {
        println!("Building Rust project...");

        let status = Command::new("cargo")
            .arg("build")
            .current_dir(output_path)
            .status()
            .context("Failed to execute cargo build")?;

        if !status.success() {
            println!("Warning: cargo build failed, you may need to run it manually");
        } else {
            println!("Rust project built successfully");
        }

        Ok(())
    }
}

impl Default for RustGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create RustGenerator")
    }
}

impl Generator for RustGenerator {
    type Params = RustParams;

    fn name(&self) -> &'static str {
        "Rust Language"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Rust language project generator")
    }

    fn get_template_path(&self) -> &'static str {
        "languages/rust"
    }

    fn generate(&mut self, params: Self::Params, output_path: &Path) -> Result<()> {
        // 验证参数
        params.validate()?;

        println!("Generating {} structure", self.name());

        // 1. 使用 cargo init 创建基础项目结构
        self.init_cargo_project(&params, output_path)?;

        // 2. 添加必要的依赖
        self.add_dependencies(output_path)?;

        // 3. 处理嵌入式模板 (在依赖添加后,这样模板可以覆盖默认文件)
        let mut template_processor = TemplateProcessor::new()?;
        let template_path = self.get_template_path();
        let context = params.to_template_context();

        // 检查嵌入式模板目录是否存在
        if crate::template_engine::embedded_template_dir_exists(template_path) {
            println!("Processing embedded templates from: {}", template_path);
            match template_processor.process_embedded_template_directory(
                template_path,
                output_path,
                context,
            ) {
                Ok(_) => println!("Embedded templates processed successfully"),
                Err(e) => {
                    eprintln!("Failed to process embedded templates: {}", e);
                    eprintln!("Error chain:");
                    for cause in e.chain() {
                        eprintln!("  - {}", cause);
                    }
                    return Err(e).context("Failed to generate Rust files");
                }
            }
        } else {
            return Err(anyhow::anyhow!(
                "{} embedded templates not found at: {}",
                self.name(),
                template_path
            ));
        }

        // 4. 构建项目
        self.build_project(output_path)?;

        println!("Rust language generation completed successfully");
        Ok(())
    }
}

impl LanguageGeneratorTrait for RustGenerator {
    fn language(&self) -> &'static str {
        Language::Rust.as_str()
    }

    fn setup_environment(&mut self, params: &Self::Params, output_path: &Path) -> Result<()> {
        // 初始化 Rust 项目
        self.init_cargo_project(params, output_path)?;

        // 添加依赖
        self.add_dependencies(output_path)?;

        Ok(())
    }

    fn generate_language_config(
        &mut self,
        params: &Self::Params,
        output_path: &Path,
    ) -> Result<()> {
        // 确保 Cargo.toml 文件存在
        let cargo_toml_path = output_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            self.init_cargo_project(params, output_path)?;
        }

        Ok(())
    }
}
