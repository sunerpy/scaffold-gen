use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use walkdir::WalkDir;

    fn ctx(project_name: &str) -> HashMap<String, Value> {
        let mut m = HashMap::new();
        m.insert("project_name".to_string(), json!(project_name));
        m.insert("project_version".to_string(), json!("0.1.0"));
        m.insert("license".to_string(), json!("MIT"));
        m
    }

    fn collect_relative_files(root: &Path) -> Vec<String> {
        WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| {
                e.path()
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect()
    }

    #[test]
    fn process_embedded_template_directory_renders_python_into_tempdir() {
        // Given: 一个临时输出目录与最小渲染上下文
        let tmp = tempfile::tempdir().expect("create tempdir");
        let mut processor = TemplateProcessor::new().expect("create processor");
        let mut context = ctx("my-cool-project");
        context.insert("python_version".to_string(), json!("3.12"));
        context.insert("package_name".to_string(), json!("my_cool_project"));
        context.insert("uv_version".to_string(), json!("0.9.1"));
        context.insert("ruff_version".to_string(), json!("0.12.1"));

        // When: 渲染嵌入式 languages/python 模板目录
        processor
            .process_embedded_template_directory("languages/python", tmp.path(), context)
            .expect("render python templates");

        // Then: 至少生成了 main.py 与 README.md（.tmpl 后缀被剥离）
        let files = collect_relative_files(tmp.path());
        assert!(
            files.iter().any(|f| f == "main.py"),
            "expected main.py to be generated, got: {files:?}"
        );
        assert!(
            files.iter().any(|f| f == "README.md"),
            "expected README.md to be generated, got: {files:?}"
        );
        assert!(
            !files.iter().any(|f| f.ends_with(".tmpl")),
            "no .tmpl files should remain, got: {files:?}"
        );
    }

    #[test]
    fn process_embedded_template_directory_leaves_no_unrendered_delimiters() {
        // Given: 临时目录 + 完整 Python 上下文
        let tmp = tempfile::tempdir().expect("create tempdir");
        let mut processor = TemplateProcessor::new().expect("create processor");
        let mut context = ctx("acme-service");
        context.insert("python_version".to_string(), json!("3.12"));
        context.insert("package_name".to_string(), json!("acme_service"));
        context.insert("uv_version".to_string(), json!("0.9.1"));
        context.insert("ruff_version".to_string(), json!("0.12.1"));

        // When: 渲染
        processor
            .process_embedded_template_directory("languages/python", tmp.path(), context)
            .expect("render python templates");

        // Then: 任何文件都不得残留自定义分隔符 `<<` 或 `%>`（证明渲染确实发生）
        for rel in collect_relative_files(tmp.path()) {
            let content = fs::read_to_string(tmp.path().join(&rel))
                .unwrap_or_else(|_| panic!("read generated file {rel}"));
            assert!(
                !content.contains("<<"),
                "file {rel} still contains unrendered variable delimiter `<<`"
            );
            assert!(
                !content.contains("%>"),
                "file {rel} still contains unrendered block delimiter `%>`"
            );
        }
    }

    #[test]
    fn process_embedded_template_directory_substitutes_project_name() {
        // Given
        let tmp = tempfile::tempdir().expect("create tempdir");
        let mut processor = TemplateProcessor::new().expect("create processor");
        let mut context = ctx("substitution-probe");
        context.insert("python_version".to_string(), json!("3.12"));
        context.insert("package_name".to_string(), json!("substitution_probe"));
        context.insert("uv_version".to_string(), json!("0.9.1"));
        context.insert("ruff_version".to_string(), json!("0.12.1"));

        // When
        processor
            .process_embedded_template_directory("languages/python", tmp.path(), context)
            .expect("render python templates");

        // Then: README.md 顶部标题应包含被替换后的 project_name
        let readme =
            fs::read_to_string(tmp.path().join("README.md")).expect("read generated README.md");
        assert!(
            readme.contains("substitution-probe"),
            "project_name was not substituted into README.md:\n{readme}"
        );
    }

    #[test]
    fn process_embedded_template_directory_renders_gin_framework() {
        // Given: gin 模板仅依赖 BaseParams 提供的变量
        let tmp = tempfile::tempdir().expect("create tempdir");
        let mut processor = TemplateProcessor::new().expect("create processor");
        let mut context = ctx("gin-app");
        context.insert("project_name_pascal".to_string(), json!("GinApp"));
        context.insert("cargo_version".to_string(), json!("0.1.0"));
        context.insert("cargo_description".to_string(), json!("a gin app"));
        context.insert("host".to_string(), json!("127.0.0.1"));
        context.insert("default_host".to_string(), json!("127.0.0.1"));
        context.insert("port".to_string(), json!(8080));
        context.insert("default_port".to_string(), json!(8080));
        context.insert("go_version".to_string(), json!("1.24"));
        context.insert("enable_swagger".to_string(), json!(true));

        // When
        processor
            .process_embedded_template_directory("frameworks/go/gin", tmp.path(), context)
            .expect("render gin templates");

        // Then: main.go 存在且无残留分隔符
        let files = collect_relative_files(tmp.path());
        assert!(
            files.iter().any(|f| f == "main.go"),
            "expected main.go, got: {files:?}"
        );
        for rel in &files {
            let content = fs::read_to_string(tmp.path().join(rel))
                .unwrap_or_else(|_| panic!("read generated file {rel}"));
            assert!(
                !content.contains("<<") && !content.contains("%>"),
                "file {rel} still contains unrendered delimiters"
            );
        }
    }
}
