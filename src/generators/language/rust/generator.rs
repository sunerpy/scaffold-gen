use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::constants::Language;
use crate::generators::core::{
    Generator, LanguageGenerator as LanguageGeneratorTrait, Parameters, TemplateProcessor,
};
use crate::generators::language::rust::parameters::RustParams;

/// Rust 语言生成器
pub struct RustGenerator {}

impl RustGenerator {
    /// 创建新的 Rust 生成器
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// 构建项目以验证依赖
    fn build_project(&self, output_path: &Path) -> Result<()> {
        println!("Building Rust workspace project...");

        let status = Command::new("cargo")
            .arg("build")
            .current_dir(output_path)
            .status()
            .context("Failed to execute cargo build")?;

        if !status.success() {
            println!("Warning: cargo build failed, you may need to run it manually");
        } else {
            println!("Rust workspace project built successfully");
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
        Some("Rust language project generator with workspace structure")
    }

    fn get_template_path(&self) -> &'static str {
        "languages/rust"
    }

    fn generate(&mut self, params: Self::Params, output_path: &Path) -> Result<()> {
        // 验证参数
        params.validate()?;

        println!("Generating {} structure with workspace", self.name());

        // 1. 处理嵌入式模板 (模板处理器会自动创建目录)
        let mut template_processor = TemplateProcessor::new()?;
        let template_path = self.get_template_path();
        let context = params.to_template_context();

        // 检查嵌入式模板目录是否存在
        if crate::template_engine::embedded_template_dir_exists(template_path) {
            println!("Processing embedded templates from: {template_path}");
            match template_processor.process_embedded_template_directory(
                template_path,
                output_path,
                context,
            ) {
                Ok(_) => println!("Embedded templates processed successfully"),
                Err(e) => {
                    eprintln!("Failed to process embedded templates: {e}");
                    eprintln!("Error chain:");
                    for cause in e.chain() {
                        eprintln!("  - {cause}");
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

        // 2. 构建项目
        self.build_project(output_path)?;

        println!("Rust language generation completed successfully");
        Ok(())
    }
}

impl LanguageGeneratorTrait for RustGenerator {
    fn language(&self) -> &'static str {
        Language::Rust.as_str()
    }

    fn setup_environment(&mut self, _params: &Self::Params, _output_path: &Path) -> Result<()> {
        // 模板处理器会自动创建目录结构
        Ok(())
    }

    fn generate_language_config(
        &mut self,
        _params: &Self::Params,
        _output_path: &Path,
    ) -> Result<()> {
        // 配置文件由模板生成
        Ok(())
    }
}
