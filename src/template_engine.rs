use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 模板引擎，负责处理Handlebars模板的渲染
pub struct TemplateEngine {
    pub handlebars: Handlebars<'static>,
    templates_dir: PathBuf,
}

impl TemplateEngine {
    /// 创建新的模板引擎实例
    pub fn new(templates_dir: PathBuf) -> Result<Self> {
        let mut handlebars = Handlebars::new();

        // 注册辅助函数
        handlebars.register_helper("to_camel_case", Box::new(to_camel_case_helper));
        handlebars.register_helper("to_snake_case", Box::new(to_snake_case_helper));

        Ok(Self {
            handlebars,
            templates_dir,
        })
    }

    /// 渲染指定的模板文件
    pub fn render_template(
        &mut self,
        template_path: &Path,
        data: &HashMap<String, Value>,
    ) -> Result<String> {
        if !template_path.exists() {
            return Err(anyhow::anyhow!(
                "Template file not found: {}\nPlease ensure the template exists",
                template_path.display()
            ));
        }

        let template_content = fs::read_to_string(template_path)
            .with_context(|| format!(
                "Failed to read template file: {}\nCheck file permissions and encoding (should be UTF-8)",
                template_path.display()
            ))?;

        self.handlebars.render_template(&template_content, data)
            .with_context(|| format!(
                "Template rendering failed for '{}'\nCommon issues:\n  - Missing template variables\n  - Invalid Handlebars syntax\n  - Circular template references",
                template_path.display()
            ))
    }

    /// 渲染框架特定的模板，支持回退机制
    #[allow(dead_code)]
    pub fn render_framework_template(
        &mut self,
        framework: &str,
        template_name: &str,
        data: &HashMap<String, Value>,
    ) -> Result<String> {
        let template_path = self.find_template_with_fallback(framework, template_name)?;
        self.render_template(&template_path, data)
    }

    /// 查找模板，支持回退机制：框架特定 -> 语言通用 -> 基础模板
    #[allow(dead_code)]
    pub fn find_template_with_fallback(
        &self,
        framework: &str,
        template_name: &str,
    ) -> Result<PathBuf> {
        let search_paths = vec![
            self.get_framework_template_path(framework, template_name),
            self.get_language_template_path("go", template_name),
            self.get_base_template_path(template_name),
        ];

        for path in &search_paths {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        // 如果都找不到，返回详细的错误信息
        let search_info = search_paths
            .iter()
            .enumerate()
            .map(|(i, path)| format!("  {}. {}", i + 1, path.display()))
            .collect::<Vec<_>>()
            .join("\n");

        Err(anyhow::anyhow!(
            "Template '{template_name}' not found. Searched in:\n{search_info}"
        ))
    }

    /// 获取框架特定模板路径
    #[allow(dead_code)]
    pub fn get_framework_template_path(&self, framework: &str, template_name: &str) -> PathBuf {
        if framework.is_empty() {
            self.get_base_template_path(template_name)
        } else {
            self.templates_dir
                .join("frameworks")
                .join("go")
                .join(framework)
                .join(template_name)
        }
    }

    /// 获取语言模板路径
    #[allow(dead_code)]
    pub fn get_language_template_path(&self, language: &str, template_name: &str) -> PathBuf {
        self.templates_dir
            .join("languages")
            .join(language)
            .join(template_name)
    }

    /// 获取基础模板路径
    #[allow(dead_code)]
    pub fn get_base_template_path(&self, template_name: &str) -> PathBuf {
        self.templates_dir.join(template_name)
    }

    /// 检查模板是否存在
    #[allow(dead_code)]
    pub fn template_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    /// 获取模板目录路径
    #[allow(dead_code)]
    pub fn get_templates_dir(&self) -> &PathBuf {
        &self.templates_dir
    }
}

// Handlebars辅助函数
fn to_camel_case_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let camel_case = to_camel_case(param);
    out.write(&camel_case)?;
    Ok(())
}

fn to_snake_case_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let snake_case = to_snake_case(param);
    out.write(&snake_case)?;
    Ok(())
}

/// 将字符串转换为驼峰命名
fn to_camel_case(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect()
}

/// 将字符串转换为蛇形命名
fn to_snake_case(s: &str) -> String {
    s.replace('-', "_").to_lowercase()
}

/// 获取模板目录路径
pub fn get_templates_dir() -> Result<PathBuf> {
    let search_paths = vec![
        // 可执行文件同级目录下的templates
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.join("templates"))),
        // 当前工作目录下的templates
        std::env::current_dir()
            .ok()
            .map(|dir| dir.join("devcli").join("templates")),
        // 相对路径
        Some(Path::new("templates").to_path_buf()),
    ];

    for path in search_paths.into_iter().flatten() {
        if path.exists() {
            return Ok(path);
        }
    }

    Err(anyhow::anyhow!("Templates directory not found"))
}
