use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::scaffold::{ParameterScope, Scaffold};
use crate::template_engine::TemplateEngine;

/// 模板处理器 - 封装模板处理的核心逻辑
pub struct TemplateProcessor {
    template_engine: TemplateEngine,
}

impl TemplateProcessor {
    /// 创建新的模板处理器
    pub fn new() -> Result<Self> {
        let templates_root = crate::template_engine::get_templates_dir()?;
        let template_engine = TemplateEngine::new(templates_root)?;

        Ok(Self { template_engine })
    }

    /// 处理嵌入式模板目录
    pub fn process_embedded_template_directory(
        &mut self,
        template_path: &str,
        output_path: &Path,
        context: HashMap<String, Value>,
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
                    let rendered_content = self
                        .template_engine
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
                } else {
                    return Err(anyhow::anyhow!("File content not found: {template_file}"));
                }
            }
        }

        Ok(())
    }

    /// 处理单个模板目录
    pub fn process_template_directory(
        &self,
        template_path: &Path,
        output_path: &Path,
        context: HashMap<String, Value>,
    ) -> Result<()> {
        // 转换为ParameterScope
        let mut params = ParameterScope::new();
        for (key, value) in context {
            params.add(&key, value);
        }

        // 使用Scaffold处理模板
        Scaffold::new(template_path)?
            .output_to(output_path)
            .with_params(params)
            .process()?
            .run_post_processors()?;

        Ok(())
    }

    /// 处理单个模板文件
    pub fn process_template_file(
        &mut self,
        template_file: &Path,
        output_file: &Path,
        context: HashMap<String, Value>,
    ) -> Result<()> {
        let rendered = self
            .template_engine
            .render_template(template_file, &context)
            .with_context(|| format!("Failed to render template: {}", template_file.display()))?;

        // 确保输出目录存在
        if let Some(parent) = output_file.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create output directory: {}", parent.display())
            })?;
        }

        std::fs::write(output_file, rendered)
            .with_context(|| format!("Failed to write output file: {}", output_file.display()))?;

        Ok(())
    }

    /// 获取模板路径
    pub fn get_template_path(&self, relative_path: &str) -> Result<PathBuf> {
        let templates_root = crate::template_engine::get_templates_dir()?;
        Ok(templates_root.join(relative_path))
    }

    /// 渲染模板内容
    pub fn render_template_content(
        &mut self,
        template_content: &str,
        context: HashMap<String, Value>,
    ) -> Result<String> {
        self.template_engine
            .render_template_content(template_content, context)
    }

    /// 检查模板是否存在（强制使用嵌入式模板）
    pub fn template_exists(&self, relative_path: &str) -> bool {
        crate::template_engine::embedded_template_exists(relative_path)
    }
}

impl Default for TemplateProcessor {
    fn default() -> Self {
        Self::new().expect("Failed to create TemplateProcessor")
    }
}
