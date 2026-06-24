use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
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

    fn should_skip_proto_gen_file(&self, relative_path: &str, params: &RustParams) -> bool {
        if !params.enable_proto_gen() {
            relative_path.starts_with("tools/proto-gen")
                || relative_path.starts_with("protos/")
                || relative_path.contains("/protos/")
        } else {
            false
        }
    }

    fn should_skip_error_gen_file(&self, relative_path: &str, params: &RustParams) -> bool {
        if !params.enable_error_gen() {
            relative_path.starts_with("tools/error-gen")
                || relative_path == "errors.toml"
                || relative_path == "errors.toml.tmpl"
        } else {
            false
        }
    }

    fn should_skip_precommit_file(&self, file_name: &str, params: &RustParams) -> bool {
        if !params.base.enable_precommit {
            file_name == ".pre-commit-config.yaml.tmpl" || file_name == ".pre-commit-config.yaml"
        } else {
            false
        }
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
        params.validate()?;

        println!("Generating {} structure with workspace", self.name());

        let mut template_processor = TemplateProcessor::new()?;
        let template_path = self.get_template_path();
        let context = params.to_template_context();

        if crate::template_engine::embedded_template_dir_exists(template_path) {
            println!("Processing embedded templates from: {template_path}");
            self.render_embedded_templates(
                &mut template_processor,
                template_path,
                output_path,
                context,
                &params,
            )?;
        } else {
            return Err(anyhow::anyhow!(
                "{} embedded templates not found at: {}",
                self.name(),
                template_path
            ));
        }

        self.build_project(output_path)?;

        println!("Rust language generation completed successfully");
        Ok(())
    }

    fn render_embedded_templates(
        &mut self,
        template_processor: &mut TemplateProcessor,
        template_path: &str,
        output_path: &Path,
        context: HashMap<String, Value>,
        params: &Self::Params,
    ) -> Result<()> {
        use std::fs;

        let template_files = crate::template_engine::get_embedded_template_files(template_path)
            .with_context(|| {
                format!("Failed to get embedded template files for: {template_path}")
            })?;

        for template_file in template_files {
            let relative_path = template_file
                .strip_prefix(&format!("{template_path}/"))
                .unwrap_or(&template_file);

            let file_name = std::path::Path::new(relative_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if self.should_skip_precommit_file(file_name, params) {
                continue;
            }

            if self.should_skip_proto_gen_file(relative_path, params) {
                continue;
            }

            if self.should_skip_error_gen_file(relative_path, params) {
                continue;
            }

            let output_relative_path = if let Some(stripped) = relative_path.strip_suffix(".tmpl") {
                stripped
            } else {
                relative_path
            };

            let output_file_path = output_path.join(output_relative_path);

            if let Some(parent) = output_file_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            if template_file.ends_with(".tmpl") {
                if let Some(template_content) =
                    crate::template_engine::get_embedded_template_content(&template_file)
                {
                    let rendered_content = match template_processor
                        .render_template_content(&template_content, context.clone())
                    {
                        Ok(content) => content,
                        Err(e) => {
                            eprintln!("❌ Template rendering error for: {template_file}");
                            eprintln!("   Error: {e:?}");
                            return Err(e).with_context(|| {
                                format!("Failed to render embedded template: {template_file}")
                            });
                        }
                    };

                    fs::write(&output_file_path, rendered_content).with_context(|| {
                        format!(
                            "Failed to write rendered file: {}",
                            output_file_path.display()
                        )
                    })?;

                    println!("📝 Rendered: {relative_path} -> {output_relative_path}");
                } else {
                    return Err(anyhow::anyhow!(
                        "Template content not found: {template_file}"
                    ));
                }
            } else if let Some(file_content) =
                crate::template_engine::get_embedded_template_content(&template_file)
            {
                fs::write(&output_file_path, file_content).with_context(|| {
                    format!("Failed to write file: {}", output_file_path.display())
                })?;

                println!("📋 Copied: {relative_path} -> {output_relative_path}");
            } else {
                return Err(anyhow::anyhow!("File content not found: {template_file}"));
            }
        }

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
