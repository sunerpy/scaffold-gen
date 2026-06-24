use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

use super::parameters::GinParams;
use crate::generators::core::{Generator, TemplateProcessor};
use crate::utils::go_tools::GoTools;

/// Gin框架级别生成器实现
#[derive(Debug)]
pub struct GinGenerator {}

impl GinGenerator {
    /// 创建新的Gin生成器
    pub fn new() -> Result<Self> {
        Ok(Self {})
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

    fn get_template_path(&self) -> &'static str {
        "frameworks/go/gin"
    }

    /// 渲染嵌入式模板 - 重写以实现Gin特定的逻辑
    fn render_embedded_templates(
        &mut self,
        template_processor: &mut TemplateProcessor,
        template_path: &str,
        output_path: &Path,
        context: HashMap<String, Value>,
        params: &Self::Params,
    ) -> Result<()> {
        use std::fs;

        // 获取嵌入式模板文件列表
        let template_files = crate::template_engine::get_embedded_template_files(template_path)
            .with_context(|| {
                format!("Failed to get embedded template files for: {template_path}")
            })?;

        for template_file in template_files {
            // 获取相对于模板路径的文件路径
            let relative_path = template_file
                .strip_prefix(&format!("{template_path}/"))
                .unwrap_or(&template_file);

            let file_name = std::path::Path::new(relative_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // 检查是否应该跳过swagger相关文件
            if self.should_skip_swagger_file(file_name, params) {
                continue;
            }

            // 检查是否应该跳过pre-commit相关文件
            if self.should_skip_precommit_file(file_name, params) {
                continue;
            }

            // 去除 .tmpl 后缀
            let output_relative_path = if let Some(stripped) = relative_path.strip_suffix(".tmpl") {
                stripped // 移除 ".tmpl"
            } else {
                relative_path
            };

            let output_file_path = output_path.join(output_relative_path);

            // 确保输出目录存在
            if let Some(parent) = output_file_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            // 判断是否为模板文件
            if template_file.ends_with(".tmpl") {
                // 获取模板内容
                if let Some(template_content) =
                    crate::template_engine::get_embedded_template_content(&template_file)
                {
                    // 渲染模板
                    let rendered_content = template_processor
                        .render_template_content(&template_content, context.clone())
                        .with_context(|| {
                            format!("Failed to render embedded template: {template_file}")
                        })?;

                    // 写入文件
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
            } else {
                // 直接复制非模板文件
                if let Some(file_content) =
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
        }

        Ok(())
    }
}

impl GinGenerator {
    /// 后处理逻辑 - 处理 Swagger 文档生成
    pub fn post_process(&self, params: &GinParams, output_path: &Path) -> Result<()> {
        if params.enable_swagger() {
            println!("Checking for swag command...");

            // 使用同步方式检查 swag 命令
            let has_swag = match std::process::Command::new("swag").arg("--version").output() {
                Ok(output) => output.status.success(),
                Err(_) => false,
            };

            if !has_swag {
                println!(
                    "Warning: 'swag' command not found. Please install swag to generate Swagger documentation:"
                );
                println!("   go install github.com/swaggo/swag/cmd/swag@latest");
                return Ok(());
            }

            // 执行 swag init 命令
            let output = std::process::Command::new("swag")
                .arg("init")
                .arg("-g")
                .arg("main.go")
                .current_dir(output_path)
                .output()
                .context("Failed to execute swag init command")?;

            if output.status.success() {
                println!("Swagger documentation generated successfully");

                // 升级 swag 版本以确保兼容性
                println!("Upgrading swag to latest version...");
                let upgrade_output = std::process::Command::new("go")
                    .arg("get")
                    .arg("-u")
                    .arg("github.com/swaggo/swag")
                    .current_dir(output_path)
                    .output()
                    .context("Failed to execute go get -u github.com/swaggo/swag command")?;

                if upgrade_output.status.success() {
                    println!("Swag upgraded successfully");
                } else {
                    let stderr = String::from_utf8_lossy(&upgrade_output.stderr);
                    println!("Warning: Failed to upgrade swag: {stderr}");
                }

                // 向 swagger.json 文件末尾添加空白行以符合格式规范
                let swagger_json_path = output_path.join("docs").join("swagger.json");
                if swagger_json_path.exists()
                    && let Ok(mut content) = std::fs::read_to_string(&swagger_json_path)
                {
                    // 检查文件是否以换行符结尾（支持不同平台的换行符）
                    let needs_newline = !content.ends_with('\n') && !content.ends_with("\r\n");

                    if needs_newline {
                        content.push('\n');

                        if let Err(e) = std::fs::write(&swagger_json_path, content) {
                            println!("Warning: Failed to add newline to swagger.json: {e}");
                        } else {
                            println!("Added newline to swagger.json for proper formatting");
                        }
                    }
                }

                // 生成 Swagger 文档后，重新运行 go mod tidy 来整理新增的依赖
                GoTools::mod_tidy(output_path)
                    .context("Failed to run go mod tidy after Swagger generation")?;
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("Failed to generate Swagger documentation: {stderr}");
            }
        }

        Ok(())
    }
}

impl GinGenerator {
    /// 检查是否应该跳过swagger相关文件
    fn should_skip_swagger_file(&self, file_name: &str, params: &GinParams) -> bool {
        if !params.enable_swagger() {
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
        if !params.enable_precommit() {
            // 如果禁用pre-commit，跳过所有pre-commit相关文件
            file_name == ".pre-commit-config.yaml.tmpl" || file_name == ".pre-commit-config.yaml"
        } else {
            false
        }
    }
}
