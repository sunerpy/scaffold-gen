use anyhow::{Context, Result};
use handlebars::Handlebars;
use include_dir::{Dir, include_dir};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 将路径标准化为Unix风格的路径分隔符
/// 这对于嵌入式模板路径是必要的，因为rust-embed使用Unix风格的路径
fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// 模板引擎，负责处理Handlebars模板的渲染
pub struct TemplateEngine {
    pub handlebars: Handlebars<'static>,
    #[allow(dead_code)]
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

    /// 渲染模板内容
    pub fn render_template_content(
        &mut self,
        template_content: &str,
        context: HashMap<String, Value>,
    ) -> Result<String> {
        let template = self
            .handlebars
            .render_template(template_content, &context)
            .context("Failed to render template content")?;
        Ok(template)
    }

    /// 渲染指定的模板文件（强制使用嵌入式模板）
    pub fn render_template(
        &mut self,
        template_path: &Path,
        data: &HashMap<String, Value>,
    ) -> Result<String> {
        let relative_path = normalize_path(&template_path.to_string_lossy());

        println!("Reading embedded template: {relative_path}");
        let template_content = read_embedded_template(&relative_path)
            .with_context(|| format!("Failed to read embedded template: {relative_path}"))?;

        println!(
            "Embedded template read successfully, content length: {}",
            template_content.len()
        );

        self.handlebars
            .render_template(&template_content, data)
            .with_context(|| {
                format!("Template rendering failed for embedded template: {relative_path}")
            })
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

// 嵌入模板目录
static EMBEDDED_TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

/// 获取模板目录路径（强制使用嵌入式模板）
pub fn get_templates_dir() -> Result<PathBuf> {
    // 直接返回空路径，因为所有模板都是嵌入式的
    Ok(PathBuf::new())
}

/// 从嵌入式模板读取文件内容
pub fn read_embedded_template(relative_path: &str) -> Result<String> {
    if let Some(file) = EMBEDDED_TEMPLATES.get_file(relative_path) {
        Ok(String::from_utf8_lossy(file.contents()).to_string())
    } else {
        Err(anyhow::anyhow!(
            "Embedded template file not found: {relative_path}"
        ))
    }
}

/// 检查嵌入式模板文件是否存在
pub fn embedded_template_exists(relative_path: &str) -> bool {
    EMBEDDED_TEMPLATES.get_file(relative_path).is_some()
}

/// 检查嵌入式模板目录是否存在
pub fn embedded_template_dir_exists(relative_path: &str) -> bool {
    if relative_path.is_empty() {
        return true; // 根目录总是存在
    }

    // 检查是否有文件以该路径开头
    for file in EMBEDDED_TEMPLATES.files() {
        let file_path = file.path().to_string_lossy();
        if file_path.starts_with(&format!("{relative_path}/")) {
            return true;
        }
    }

    // 递归检查子目录
    fn check_dir_recursive(dir: &Dir, target_path: &str, current_path: &str) -> bool {
        if current_path == target_path {
            return true;
        }

        for subdir in dir.dirs() {
            let subdir_name = subdir.path().file_name().unwrap().to_string_lossy();
            let subdir_path = if current_path.is_empty() {
                subdir_name.to_string()
            } else {
                format!("{current_path}/{subdir_name}")
            };

            if check_dir_recursive(subdir, target_path, &subdir_path) {
                return true;
            }
        }

        false
    }

    check_dir_recursive(&EMBEDDED_TEMPLATES, relative_path, "")
}

/// 获取嵌入式模板内容
pub fn get_embedded_template_content(relative_path: &str) -> Option<String> {
    EMBEDDED_TEMPLATES
        .get_file(relative_path)
        .map(|file| String::from_utf8_lossy(file.contents()).to_string())
}

/// 获取嵌入式模板目录中的所有文件
pub fn get_embedded_template_files(relative_path: &str) -> Result<Vec<String>> {
    fn collect_files_recursive(dir: &Dir, current_path: &str, files: &mut Vec<String>) {
        for file in dir.files() {
            let file_path = if current_path.is_empty() {
                file.path().to_string_lossy().to_string()
            } else {
                format!(
                    "{}/{}",
                    current_path,
                    file.path().file_name().unwrap().to_string_lossy()
                )
            };
            files.push(normalize_path(&file_path));
        }

        for subdir in dir.dirs() {
            let subdir_name = subdir.path().file_name().unwrap().to_string_lossy();
            let subdir_path = if current_path.is_empty() {
                subdir_name.to_string()
            } else {
                format!("{current_path}/{subdir_name}")
            };
            collect_files_recursive(subdir, &subdir_path, files);
        }
    }

    let mut all_files = Vec::new();
    collect_files_recursive(&EMBEDDED_TEMPLATES, "", &mut all_files);

    // 如果指定了相对路径，过滤出该路径下的文件
    if relative_path.is_empty() {
        Ok(all_files)
    } else {
        let filtered_files: Vec<String> = all_files
            .into_iter()
            .filter(|file| {
                file.starts_with(relative_path)
                    && file.len() > relative_path.len()
                    && file.chars().nth(relative_path.len()) == Some('/')
            })
            .collect();
        Ok(filtered_files)
    }
}
