use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

use super::parameters::GinParams;
use crate::constants::{Framework, Language};
use crate::generators::core::{
    FrameworkGenerator as FrameworkGeneratorTrait, Generator, TemplateProcessor,
};
use crate::utils::go_tools::GoTools;

/// Gin框架级别生成器实现
pub struct GinGenerator {
    template_processor: TemplateProcessor,
}

impl GinGenerator {
    /// 创建新的Gin生成器
    pub fn new() -> Result<Self> {
        Ok(Self {
            template_processor: TemplateProcessor::new()?,
        })
    }
}

impl Default for GinGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create GinGenerator")
    }
}

impl Generator for GinGenerator {
    type Params = GinParams;

    fn name(&self) -> &'static str {
        "Gin"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Generates Gin web framework specific files and structure")
    }

    fn get_template_path(&self) -> &'static str {
        "frameworks/go/gin"
    }

    /// 重写模板渲染方法以支持 Swagger 文件过滤
    fn render_templates(
        &mut self,
        template_processor: &TemplateProcessor,
        template_path: &str,
        output_path: &Path,
        context: HashMap<String, Value>,
        params: &Self::Params,
    ) -> Result<()> {
        use std::fs;

        // 获取模板的绝对路径
        let template_path_obj = template_processor.get_template_path(template_path)?;

        println!(
            "🔍 Processing template directory: {}",
            template_path_obj.display()
        );

        for entry in WalkDir::new(&template_path_obj) {
            let entry =
                entry.map_err(|e| anyhow::anyhow!("Failed to read directory entry: {e}"))?;
            let path = entry.path();

            if path.is_file() {
                let relative_path = path.strip_prefix(&template_path_obj)?;
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // 检查是否应该跳过swagger相关文件
                if self.should_skip_swagger_file(file_name, params) {
                    println!("⏭️  Skipping swagger file: {file_name}");
                    continue;
                }

                // 检查是否应该跳过pre-commit相关文件
                if self.should_skip_precommit_file(file_name, params) {
                    println!("⏭️  Skipping pre-commit file: {file_name}");
                    continue;
                }

                // 去除 .tmpl 后缀
                let output_relative_path =
                    if relative_path.extension().and_then(|s| s.to_str()) == Some("tmpl") {
                        relative_path.with_extension("")
                    } else {
                        relative_path.to_path_buf()
                    };

                let output_file_path = output_path.join(&output_relative_path);

                // 确保输出目录存在
                if let Some(parent) = output_file_path.parent() {
                    fs::create_dir_all(parent).with_context(|| {
                        format!("Failed to create directory: {}", parent.display())
                    })?;
                }

                // 判断是否为模板文件
                if path.extension().and_then(|s| s.to_str()) == Some("tmpl") {
                    // 处理模板文件 - 使用实例的模板处理器
                    self.template_processor
                        .process_template_file(path, &output_file_path, context.clone())
                        .with_context(|| {
                            format!("Failed to render template: {}", path.display())
                        })?;

                    println!(
                        "📝 Rendered: {} -> {}",
                        relative_path.display(),
                        output_relative_path.display()
                    );
                } else {
                    // 直接复制非模板文件
                    fs::copy(path, &output_file_path).with_context(|| {
                        format!(
                            "Failed to copy file: {} -> {}",
                            path.display(),
                            output_file_path.display()
                        )
                    })?;

                    println!(
                        "📋 Copied: {} -> {}",
                        relative_path.display(),
                        output_relative_path.display()
                    );
                }
            }
        }

        Ok(())
    }

    /// 后处理逻辑 - 处理 Swagger 文档生成
    fn post_process(&mut self, params: &Self::Params, output_path: &Path) -> Result<()> {
        if params.enable_swagger {
            println!("🔍 Checking for swag command...");

            // 使用同步方式检查 swag 命令
            let has_swag = match std::process::Command::new("swag").arg("--version").output() {
                Ok(output) => output.status.success(),
                Err(_) => false,
            };

            if !has_swag {
                println!(
                    "⚠️  Warning: 'swag' command not found. Please install swag to generate Swagger documentation:"
                );
                println!("   go install github.com/swaggo/swag/cmd/swag@latest");
                return Ok(());
            }

            println!("✅ Found swag command, generating Swagger documentation...");

            // 执行 swag init 命令
            let output = std::process::Command::new("swag")
                .arg("init")
                .arg("-g")
                .arg("main.go")
                .current_dir(output_path)
                .output()
                .context("Failed to execute swag init command")?;

            if output.status.success() {
                println!("✅ Swagger documentation generated successfully");

                // 生成 Swagger 文档后，重新运行 go mod tidy 来整理新增的依赖
                println!("🔧 Updating dependencies after Swagger generation...");
                GoTools::mod_tidy(output_path)
                    .context("Failed to run go mod tidy after Swagger generation")?;
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("❌ Failed to generate Swagger documentation: {stderr}");
            }
        }

        Ok(())
    }
}

impl GinGenerator {
    /// 检查是否应该跳过swagger相关文件
    fn should_skip_swagger_file(&self, file_name: &str, params: &GinParams) -> bool {
        if !params.enable_swagger {
            // 如果禁用swagger，跳过所有swagger相关文件
            file_name.contains("swagger")
                || file_name.starts_with("docs.go")
                || file_name.ends_with("swagger.json.tmpl")
                || file_name.ends_with("swagger.yaml.tmpl")
        } else {
            false
        }
    }

    /// 检查是否应该跳过pre-commit相关文件
    fn should_skip_precommit_file(&self, file_name: &str, params: &GinParams) -> bool {
        if !params.enable_precommit {
            // 如果禁用pre-commit，跳过所有pre-commit相关文件
            file_name == ".pre-commit-config.yaml.tmpl" || file_name == ".pre-commit-config.yaml"
        } else {
            false
        }
    }
}

impl FrameworkGeneratorTrait for GinGenerator {
    fn framework(&self) -> &'static str {
        Framework::Gin.as_str()
    }

    fn language(&self) -> &'static str {
        Language::Go.as_str()
    }

    fn generate_basic_structure(
        &mut self,
        _params: &Self::Params,
        _output_path: &Path,
    ) -> Result<()> {
        // 不再需要自定义结构生成，完全依赖模板
        Ok(())
    }

    fn generate_config(&mut self, _params: &Self::Params, _output_path: &Path) -> Result<()> {
        // 配置文件通过模板生成
        Ok(())
    }

    fn generate_middleware(&mut self, _params: &Self::Params, _output_path: &Path) -> Result<()> {
        // 中间件通过模板生成
        Ok(())
    }
}
