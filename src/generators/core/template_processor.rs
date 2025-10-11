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

    /// 检查模板是否存在
    pub fn template_exists(&self, relative_path: &str) -> bool {
        if let Ok(path) = self.get_template_path(relative_path) {
            path.exists()
        } else {
            false
        }
    }
}

impl Default for TemplateProcessor {
    fn default() -> Self {
        Self::new().expect("Failed to create TemplateProcessor")
    }
}
