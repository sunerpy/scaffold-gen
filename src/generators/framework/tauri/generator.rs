use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use super::parameters::TauriParams;
use crate::constants::{Framework, Language};
use crate::generators::core::{
    FrameworkGenerator as FrameworkGeneratorTrait, Generator, TemplateProcessor,
};

/// Tauri框架级别生成器实现
#[derive(Debug)]
pub struct TauriGenerator {}

impl TauriGenerator {
    /// 创建新的Tauri生成器
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// 检查 create-tauri-app 是否已安装
    pub fn check_create_tauri_app() -> Result<bool> {
        let output = Command::new("cargo")
            .args(["install", "--list"])
            .output()
            .context("Failed to execute cargo install --list")?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.contains("create-tauri-app"))
        } else {
            Ok(false)
        }
    }

    /// 检查 pnpm 是否已安装
    pub fn check_pnpm() -> Result<bool> {
        match Command::new("pnpm").arg("--version").output() {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    /// 安装 create-tauri-app
    pub fn install_create_tauri_app() -> Result<()> {
        println!("📦 Installing create-tauri-app...");
        let output = Command::new("cargo")
            .args(["install", "create-tauri-app"])
            .output()
            .context("Failed to install create-tauri-app")?;

        if output.status.success() {
            println!("✅ create-tauri-app installed successfully");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!(
                "Failed to install create-tauri-app: {stderr}"
            ))
        }
    }

    /// 使用 create-tauri-app 创建项目
    pub fn create_tauri_project(project_name: &str, output_path: &Path) -> Result<()> {
        println!("🚀 Creating Tauri project with create-tauri-app...");

        // 获取父目录
        let parent_dir = output_path.parent().unwrap_or_else(|| Path::new("."));

        // 使用 cargo create-tauri-app 创建项目
        // 使用非交互模式，指定模板
        let output = Command::new("cargo")
            .args([
                "create-tauri-app",
                project_name,
                "--template",
                "vue-ts",
                "--manager",
                "pnpm",
                "--yes",
            ])
            .current_dir(parent_dir)
            .output()
            .context("Failed to execute cargo create-tauri-app")?;

        if output.status.success() {
            println!("✅ Tauri project created successfully");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(anyhow::anyhow!(
                "Failed to create Tauri project:\nstdout: {stdout}\nstderr: {stderr}"
            ))
        }
    }

    /// 安装前端依赖
    pub fn install_dependencies(output_path: &Path) -> Result<()> {
        println!("📦 Installing frontend dependencies...");

        let output = Command::new("pnpm")
            .arg("install")
            .current_dir(output_path)
            .output()
            .context("Failed to execute pnpm install")?;

        if output.status.success() {
            println!("✅ Dependencies installed successfully");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("⚠️ Warning: Failed to install dependencies: {stderr}");
            // 不返回错误，让用户手动安装
            Ok(())
        }
    }

    /// 检查是否应该跳过pre-commit相关文件
    fn should_skip_precommit_file(&self, file_name: &str, params: &TauriParams) -> bool {
        if !params.enable_precommit() {
            file_name == ".pre-commit-config.yaml.tmpl" || file_name == ".pre-commit-config.yaml"
        } else {
            false
        }
    }

    /// 检查是否应该跳过proto-gen相关文件
    fn should_skip_proto_gen_file(&self, relative_path: &str, params: &TauriParams) -> bool {
        if !params.enable_proto_gen() {
            relative_path.starts_with("tools/proto-gen")
                || relative_path.starts_with("protos/")
                || relative_path.contains("/protos/")
        } else {
            false
        }
    }

    fn should_skip_error_gen_file(&self, relative_path: &str, params: &TauriParams) -> bool {
        if !params.enable_error_gen() {
            relative_path.starts_with("tools/error-gen")
                || relative_path == "errors.toml"
                || relative_path == "errors.toml.tmpl"
        } else {
            false
        }
    }
}

impl Default for TauriGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create TauriGenerator")
    }
}

impl Generator for TauriGenerator {
    type Params = TauriParams;

    fn name(&self) -> &'static str {
        "Tauri"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Generates Tauri desktop application with Vue.js frontend")
    }

    fn get_template_path(&self) -> &'static str {
        "frameworks/rust/tauri"
    }

    /// 渲染嵌入式模板 - 重写以实现Tauri特定的逻辑
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

            // 检查是否应该跳过pre-commit相关文件
            if self.should_skip_precommit_file(file_name, params) {
                continue;
            }

            // 检查是否应该跳过proto-gen相关文件
            if self.should_skip_proto_gen_file(relative_path, params) {
                continue;
            }

            // 检查是否应该跳过error-gen相关文件
            if self.should_skip_error_gen_file(relative_path, params) {
                continue;
            }

            // 去除 .tmpl 后缀
            let output_relative_path = if let Some(stripped) = relative_path.strip_suffix(".tmpl") {
                stripped
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
                    let rendered_content = match template_processor
                        .render_template_content(&template_content, context.clone())
                    {
                        Ok(content) => content,
                        Err(e) => {
                            eprintln!("❌ Template rendering error for: {template_file}");
                            eprintln!("   Error: {e:?}");
                            eprintln!(
                                "   Template preview: {}...",
                                &template_content.chars().take(300).collect::<String>()
                            );
                            return Err(e).with_context(|| {
                                format!("Failed to render embedded template: {template_file}")
                            });
                        }
                    };

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

impl FrameworkGeneratorTrait for TauriGenerator {
    fn framework(&self) -> &'static str {
        Framework::Tauri.as_str()
    }

    fn language(&self) -> &'static str {
        Language::Rust.as_str()
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
        // Tauri 不需要中间件
        Ok(())
    }
}
