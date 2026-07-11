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
fn render_template_reads_embedded_by_path_and_substitutes() {
    let eng = engine();
    let data = ctx(&[("project_name", json!("path-render"))]);
    let out = eng
        .render_template(Path::new("languages/python/main.py.tmpl"), &data)
        .expect("render embedded template by path");
    assert!(out.contains("path-render"));
    assert!(!out.contains("<<"));
}

#[test]
fn render_template_errors_on_missing_embedded_path() {
    let eng = engine();
    let data = HashMap::new();
    let err = eng
        .render_template(Path::new("languages/python/nope.tmpl"), &data)
        .expect_err("missing embedded template should error");
    let msg = format!("{err:#}");
    assert!(msg.contains("Failed to read embedded template"));
}

#[test]
fn read_embedded_template_ok_and_err() {
    assert!(read_embedded_template("languages/python/main.py.tmpl").is_ok());
    let err = read_embedded_template("nonexistent/file.tmpl").expect_err("missing errors");
    assert!(format!("{err:#}").contains("Embedded template file not found"));
}

#[test]
fn to_camel_case_handles_empty_segments() {
    assert_eq!(to_camel_case("my--project"), "MyProject");
    assert_eq!(to_camel_case(""), "");
    assert_eq!(to_camel_case("-lead"), "Lead");
}

#[test]
fn get_embedded_template_files_root_returns_all() {
    let all = get_embedded_template_files("").expect("list all files");
    assert!(!all.is_empty());
    assert!(all.iter().any(|f| f == "languages/python/main.py.tmpl"));
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
