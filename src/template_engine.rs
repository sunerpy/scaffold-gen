use anyhow::{Context, Result};
use include_dir::{Dir, include_dir};
use minijinja::{Environment, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// 模板引擎 - 使用自定义分隔符 << >> 避免与 JS/TS 的 {{ }} 冲突
pub struct TemplateEngine {
    env: Environment<'static>,
    #[allow(dead_code)]
    templates_dir: PathBuf,
}

impl TemplateEngine {
    pub fn new(templates_dir: PathBuf) -> Result<Self> {
        let mut env = Environment::new();

        // 自定义分隔符: <<var>>, <%if%>, <#comment#>
        env.set_syntax(
            minijinja::syntax::SyntaxConfig::builder()
                .variable_delimiters("<<", ">>")
                .block_delimiters("<%", "%>")
                .comment_delimiters("<#", "#>")
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to configure syntax: {}", e))?,
        );

        env.add_filter("to_camel_case", to_camel_case_filter);
        env.add_filter("to_snake_case", to_snake_case_filter);

        Ok(Self { env, templates_dir })
    }

    pub fn render_template_content(
        &self,
        template_content: &str,
        context: HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        let ctx = Value::from_serialize(&context);
        self.env
            .render_str(template_content, ctx)
            .context("Failed to render template content")
    }

    pub fn render_template(
        &self,
        template_path: &Path,
        data: &HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        let relative_path = normalize_path(&template_path.to_string_lossy());

        println!("Reading embedded template: {relative_path}");
        let template_content = read_embedded_template(&relative_path)
            .with_context(|| format!("Failed to read embedded template: {relative_path}"))?;

        println!(
            "Embedded template read successfully, content length: {}",
            template_content.len()
        );

        let ctx = Value::from_serialize(data);
        self.env
            .render_str(&template_content, ctx)
            .with_context(|| {
                format!("Template rendering failed for embedded template: {relative_path}")
            })
    }
}

fn to_camel_case_filter(value: &str) -> String {
    to_camel_case(value)
}

fn to_snake_case_filter(value: &str) -> String {
    to_snake_case(value)
}

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

fn to_snake_case(s: &str) -> String {
    s.replace('-', "_").to_lowercase()
}

static EMBEDDED_TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

pub fn get_templates_dir() -> Result<PathBuf> {
    Ok(PathBuf::new())
}

pub fn read_embedded_template(relative_path: &str) -> Result<String> {
    if let Some(file) = EMBEDDED_TEMPLATES.get_file(relative_path) {
        Ok(String::from_utf8_lossy(file.contents()).to_string())
    } else {
        Err(anyhow::anyhow!(
            "Embedded template file not found: {relative_path}"
        ))
    }
}

pub fn embedded_template_exists(relative_path: &str) -> bool {
    EMBEDDED_TEMPLATES.get_file(relative_path).is_some()
}

pub fn embedded_template_dir_exists(relative_path: &str) -> bool {
    if relative_path.is_empty() {
        return true;
    }

    for file in EMBEDDED_TEMPLATES.files() {
        let file_path = file.path().to_string_lossy();
        if file_path.starts_with(&format!("{relative_path}/")) {
            return true;
        }
    }

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

pub fn get_embedded_template_content(relative_path: &str) -> Option<String> {
    EMBEDDED_TEMPLATES
        .get_file(relative_path)
        .map(|file| String::from_utf8_lossy(file.contents()).to_string())
}

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

    if relative_path.is_empty() {
        Ok(all_files)
    } else {
        let normalized_prefix = normalize_path(relative_path);
        let prefix_with_slash = format!("{normalized_prefix}/");

        let filtered_files: Vec<String> = all_files
            .into_iter()
            .filter(|file| file.starts_with(&prefix_with_slash))
            .collect();
        Ok(filtered_files)
    }
}
