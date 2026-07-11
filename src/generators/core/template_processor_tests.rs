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
fn process_template_file_renders_single_embedded_template() {
    // Given: 单个嵌入模板文件路径 + 临时输出文件
    let tmp = tempfile::tempdir().expect("create tempdir");
    let mut processor = TemplateProcessor::new().expect("create processor");
    let out = tmp.path().join("nested").join("main.py");
    let mut context = ctx("single-file-app");
    context.insert("package_name".to_string(), json!("single_file_app"));

    // When: 按路径渲染 languages/python/main.py.tmpl
    processor
        .process_template_file(Path::new("languages/python/main.py.tmpl"), &out, context)
        .expect("render single template file");

    // Then: 输出文件被创建（父目录自动建立），内容含替换后的 project_name，无残留分隔符
    let content = fs::read_to_string(&out).expect("read rendered file");
    assert!(content.contains("single-file-app"));
    assert!(!content.contains("<<"));
}

#[test]
fn get_template_path_joins_relative_onto_root() {
    let processor = TemplateProcessor::new().expect("create processor");
    let path = processor
        .get_template_path("languages/python/main.py.tmpl")
        .expect("resolve template path");
    assert!(path.ends_with("languages/python/main.py.tmpl"));
}

#[test]
fn render_template_content_substitutes_variable() {
    let mut processor = TemplateProcessor::new().expect("create processor");
    let context = ctx("render-probe");
    let out = processor
        .render_template_content("name = <<project_name>>", context)
        .expect("render content");
    assert_eq!(out, "name = render-probe");
}

#[test]
fn template_exists_reports_presence() {
    let processor = TemplateProcessor::new().expect("create processor");
    assert!(processor.template_exists("languages/python/main.py.tmpl"));
    assert!(!processor.template_exists("languages/python/definitely-missing.tmpl"));
}

#[test]
fn process_embedded_template_directory_copies_non_template_files_verbatim() {
    // Given: 一个只含非 .tmpl 文件的嵌入目录（tauri capabilities/default.json）
    let tmp = tempfile::tempdir().expect("create tempdir");
    let mut processor = TemplateProcessor::new().expect("create processor");

    // When: 处理该目录 —— 非 .tmpl 文件应原样复制
    processor
        .process_embedded_template_directory(
            "frameworks/rust/tauri/src-tauri/capabilities",
            tmp.path(),
            HashMap::new(),
        )
        .expect("copy non-template files");

    // Then: default.json 被复制，且内容与嵌入源逐字节一致
    let copied = fs::read_to_string(tmp.path().join("default.json")).expect("read copied file");
    let source = crate::template_engine::get_embedded_template_content(
        "frameworks/rust/tauri/src-tauri/capabilities/default.json",
    )
    .expect("embedded source present");
    assert_eq!(copied, source);
}

#[test]
fn process_embedded_template_directory_creates_nested_directory_structure() {
    // Given: gin 模板含嵌套目录（如 handlers/、routers/）
    let tmp = tempfile::tempdir().expect("create tempdir");
    let mut processor = TemplateProcessor::new().expect("create processor");
    let mut context = ctx("nested-gin");
    context.insert("project_name_pascal".to_string(), json!("NestedGin"));
    context.insert("host".to_string(), json!("127.0.0.1"));
    context.insert("default_host".to_string(), json!("127.0.0.1"));
    context.insert("port".to_string(), json!(8080));
    context.insert("default_port".to_string(), json!(8080));
    context.insert("go_version".to_string(), json!("1.24"));
    context.insert("enable_swagger".to_string(), json!(false));

    // When
    processor
        .process_embedded_template_directory("frameworks/go/gin", tmp.path(), context)
        .expect("render gin templates");

    // Then: 至少一个生成文件位于子目录中（相对路径含 '/'），证明嵌套结构被保留
    let files = collect_relative_files(tmp.path());
    assert!(
        files.iter().any(|f| f.contains('/')),
        "expected at least one file in a nested directory, got: {files:?}"
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
