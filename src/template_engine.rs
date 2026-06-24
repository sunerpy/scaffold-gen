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
}

impl TemplateEngine {
    pub fn new(_templates_dir: PathBuf) -> Result<Self> {
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

        Ok(Self { env })
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

        tracing::debug!("Reading embedded template: {relative_path}");
        let template_content = read_embedded_template(&relative_path)
            .with_context(|| format!("Failed to read embedded template: {relative_path}"))?;

        tracing::debug!(
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
            let subdir_name = subdir
                .path()
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| subdir.path().to_string_lossy().to_string());
            let subdir_path = if current_path.is_empty() {
                subdir_name
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
                let file_name = file
                    .path()
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| file.path().to_string_lossy().to_string());
                format!("{current_path}/{file_name}")
            };
            files.push(normalize_path(&file_path));
        }

        for subdir in dir.dirs() {
            let subdir_name = subdir
                .path()
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| subdir.path().to_string_lossy().to_string());
            let subdir_path = if current_path.is_empty() {
                subdir_name
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn engine() -> TemplateEngine {
        TemplateEngine::new(PathBuf::new()).expect("create engine")
    }

    fn ctx(pairs: &[(&str, serde_json::Value)]) -> HashMap<String, serde_json::Value> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), v.clone()))
            .collect()
    }

    #[test]
    fn renders_variable_with_double_angle_delimiters() {
        // Given: 一个使用 <<var>> 变量分隔符的模板
        let eng = engine();
        let context = ctx(&[("project_name", json!("my-app"))]);

        // When: 渲染
        let out = eng
            .render_template_content("name = <<project_name>>", context)
            .expect("render");

        // Then: 变量被替换
        assert_eq!(out, "name = my-app");
    }

    #[test]
    fn renders_block_with_percent_delimiters() {
        // Given: 一个使用 <% if %> 块分隔符的模板
        let eng = engine();
        let context = ctx(&[("enable_swagger", json!(true))]);

        // When
        let out = eng
            .render_template_content("<% if enable_swagger %>SWAGGER<% endif %>", context)
            .expect("render");

        // Then: 块按条件渲染
        assert_eq!(out, "SWAGGER");
    }

    #[test]
    fn block_false_branch_is_omitted() {
        // Given
        let eng = engine();
        let context = ctx(&[("enable_swagger", json!(false))]);

        // When
        let out = eng
            .render_template_content("A<% if enable_swagger %>X<% endif %>B", context)
            .expect("render");

        // Then
        assert_eq!(out, "AB");
    }

    #[test]
    fn strips_comment_with_hash_delimiters() {
        // Given: 一个使用 <# comment #> 注释分隔符的模板
        let eng = engine();

        // When
        let out = eng
            .render_template_content("a<# this is a comment #>b", HashMap::new())
            .expect("render");

        // Then: 注释被剥离
        assert_eq!(out, "ab");
    }

    #[test]
    fn undefined_variable_renders_empty() {
        // Given: 默认 Lenient 行为下未定义变量渲染为空
        let eng = engine();

        // When
        let out = eng
            .render_template_content("x=<<missing>>y", HashMap::new())
            .expect("render");

        // Then
        assert_eq!(out, "x=y");
    }

    #[test]
    fn to_camel_case_filter_capitalizes_hyphen_segments() {
        // Given: to_camel_case 过滤器按 `-` 分段并首字母大写
        let eng = engine();
        let context = ctx(&[("name", json!("my-cool-project"))]);

        // When
        let out = eng
            .render_template_content("<<name | to_camel_case>>", context)
            .expect("render");

        // Then
        assert_eq!(out, "MyCoolProject");
    }

    #[test]
    fn to_snake_case_filter_replaces_hyphens() {
        // Given: to_snake_case 过滤器将 `-` 替换为 `_` 并转小写
        let eng = engine();
        let context = ctx(&[("name", json!("My-Cool-Project"))]);

        // When
        let out = eng
            .render_template_content("<<name | to_snake_case>>", context)
            .expect("render");

        // Then
        assert_eq!(out, "my_cool_project");
    }

    #[test]
    fn to_camel_case_fn_matches_current_behavior() {
        assert_eq!(to_camel_case("my-cool-project"), "MyCoolProject");
        assert_eq!(to_camel_case("single"), "Single");
        assert_eq!(to_camel_case("ALL-CAPS"), "AllCaps");
    }

    #[test]
    fn to_snake_case_fn_matches_current_behavior() {
        assert_eq!(to_snake_case("My-Cool-Project"), "my_cool_project");
        assert_eq!(to_snake_case("Already_Snake"), "already_snake");
        assert_eq!(to_snake_case("single"), "single");
    }

    #[test]
    fn embedded_python_main_template_exists() {
        assert!(embedded_template_exists("languages/python/main.py.tmpl"));
        assert!(!embedded_template_exists("languages/python/nope.tmpl"));
    }

    #[test]
    fn embedded_template_dir_exists_for_known_dirs() {
        assert!(embedded_template_dir_exists("languages/python"));
        assert!(embedded_template_dir_exists("frameworks/go/gin"));
        assert!(embedded_template_dir_exists(""));
        assert!(!embedded_template_dir_exists("languages/does-not-exist"));
    }

    #[test]
    fn get_embedded_template_files_filters_by_prefix_and_includes_main() {
        let files = get_embedded_template_files("languages/python").expect("list files");
        assert!(!files.is_empty());
        assert!(files.iter().all(|f| f.starts_with("languages/python/")));
        assert!(files.iter().any(|f| f == "languages/python/main.py.tmpl"));
    }

    #[test]
    fn get_embedded_template_content_returns_some_for_known_file() {
        let content = get_embedded_template_content("languages/python/main.py.tmpl");
        assert!(content.is_some());
        assert!(content.unwrap().contains("<<project_name>>"));
        assert!(get_embedded_template_content("languages/python/missing.tmpl").is_none());
    }
}
