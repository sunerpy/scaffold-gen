use std::fs;
use std::path::Path;

use scaffold_gen::generators::core::{Parameters, TemplateProcessor};
use scaffold_gen::generators::language::python::PythonParams;
use walkdir::WalkDir;

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
fn python_embedded_generation_renders_without_external_tools() {
    // Given: 通过公开 API 构造的真实 PythonParams 与其完整模板上下文
    let params = PythonParams::new("integration-demo".to_string());
    let context = params.to_template_context();

    let tmp = tempfile::tempdir().expect("create tempdir");
    let mut processor = TemplateProcessor::new().expect("create processor");

    // When: 走 live 渲染路径（process_embedded_template_directory），不触发 uv 构建
    processor
        .process_embedded_template_directory("languages/python", tmp.path(), context)
        .expect("render embedded python templates");

    // Then(1): 预期文件存在且 .tmpl 后缀已剥离
    let files = collect_relative_files(tmp.path());
    assert!(
        files.iter().any(|f| f == "main.py"),
        "expected main.py, got: {files:?}"
    );
    assert!(
        files.iter().any(|f| f == "README.md"),
        "expected README.md, got: {files:?}"
    );
    assert!(
        !files.iter().any(|f| f.ends_with(".tmpl")),
        "no .tmpl files should remain, got: {files:?}"
    );

    // Then(2): 任何文件都不得残留自定义分隔符 `<<` 或 `%>`
    for rel in &files {
        let content = fs::read_to_string(tmp.path().join(rel))
            .unwrap_or_else(|_| panic!("read generated file {rel}"));
        assert!(
            !content.contains("<<"),
            "file {rel} still contains unrendered `<<`"
        );
        assert!(
            !content.contains("%>"),
            "file {rel} still contains unrendered `%>`"
        );
    }

    // Then(3): project_name 被替换进 README
    let readme =
        fs::read_to_string(tmp.path().join("README.md")).expect("read generated README.md");
    assert!(
        readme.contains("integration-demo"),
        "project_name not substituted into README.md:\n{readme}"
    );
}

#[test]
fn fastapi_embedded_generation_renders_without_external_tools() {
    // Given: 一个带 host/port 的 PythonParams（FastAPI 复用 Python 上下文）
    let mut params = PythonParams::new("fastapi-demo".to_string());
    params.base.host = Some("0.0.0.0".to_string());
    params.base.port = Some(8000);
    let context = params.to_template_context();

    let tmp = tempfile::tempdir().expect("create tempdir");
    let mut processor = TemplateProcessor::new().expect("create processor");

    // When: 走 live 渲染路径渲染 frameworks/python/fastapi（不触发 uv/uvicorn）
    processor
        .process_embedded_template_directory("frameworks/python/fastapi", tmp.path(), context)
        .expect("render embedded fastapi templates");

    // Then(1): 关键文件存在且 .tmpl 后缀已剥离
    let files = collect_relative_files(tmp.path());
    for expected in [
        "pyproject.toml",
        "main.py",
        "config.toml",
        "README.md",
        "app/main.py",
        "app/settings.py",
        "app/routes/example.py",
        "app/routes/health.py",
    ] {
        assert!(
            files.iter().any(|f| f == expected),
            "expected {expected}, got: {files:?}"
        );
    }
    assert!(
        !files.iter().any(|f| f.ends_with(".tmpl")),
        "no .tmpl files should remain, got: {files:?}"
    );

    // Then(2): 任何文件都不得残留自定义分隔符 `<<` 或 `%>`
    for rel in &files {
        let content = fs::read_to_string(tmp.path().join(rel))
            .unwrap_or_else(|_| panic!("read generated file {rel}"));
        assert!(
            !content.contains("<<"),
            "file {rel} still contains unrendered `<<`"
        );
        assert!(
            !content.contains("%>"),
            "file {rel} still contains unrendered `%>`"
        );
    }

    // Then(3): project_name 被替换进 README，host/port 被写入 config.toml
    let readme =
        fs::read_to_string(tmp.path().join("README.md")).expect("read generated README.md");
    assert!(
        readme.contains("fastapi-demo"),
        "project_name not substituted into README.md:\n{readme}"
    );
    let config = fs::read_to_string(tmp.path().join("config.toml")).expect("read config.toml");
    assert!(
        config.contains("0.0.0.0") && config.contains("8000"),
        "host/port not driven into config.toml:\n{config}"
    );
}

#[test]
fn vue3_embedded_generation_renders_without_external_tools() {
    // 1. Arrange: setup temp dir and parameters
    let temp_dir = tempfile::tempdir().unwrap();
    let project_name = "test-vue3-embedded".to_string();
    let params = scaffold_gen::generators::framework::vue3::Vue3Params::from_project_name(
        project_name.clone(),
    );

    // 2. Act: run the template processor
    let mut processor = scaffold_gen::generators::core::TemplateProcessor::new()
        .expect("Failed to initialize template processor");

    // 模拟 orchestrator 里的流程
    use scaffold_gen::generators::core::Parameters;
    let context = params.to_template_context();

    processor
        .process_embedded_template_directory("frameworks/typescript/vue3", temp_dir.path(), context)
        .expect("Failed to process Vue3 embedded templates");

    // 3. Assert: check that the expected files exist and project_name is substituted
    let package_json = temp_dir.path().join("package.json");
    assert!(package_json.exists(), "package.json was not generated");

    let package_json_content = std::fs::read_to_string(&package_json).unwrap();
    assert!(
        package_json_content.contains("\"name\": \"test-vue3-embedded\""),
        "package.json did not contain substituted project name"
    );

    let vite_config = temp_dir.path().join("vite.config.ts");
    assert!(vite_config.exists(), "vite.config.ts was not generated");

    let main_ts = temp_dir.path().join("src/main.ts");
    assert!(main_ts.exists(), "src/main.ts was not generated");

    let app_vue = temp_dir.path().join("src/App.vue");
    assert!(app_vue.exists(), "src/App.vue was not generated");

    let index_html = temp_dir.path().join("index.html");
    assert!(index_html.exists(), "index.html was not generated");

    let readme = temp_dir.path().join("README.md");
    assert!(readme.exists(), "README.md was not generated");
    let readme_content = std::fs::read_to_string(&readme).unwrap();
    assert!(
        readme_content.contains("# test-vue3-embedded"),
        "README.md did not contain substituted project name"
    );

    // Do NOT assert no unrendered delimiters `<<` or `>>` for Vue templates,
    // because Vue uses `{{` which is not the engine delimiter anyway,
    // but there might be other things. Just ensuring no `<<` is fine since
    // Vue uses `{{}}`, not `<<>>`.
    for entry in walkdir::WalkDir::new(temp_dir.path())
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if !content.is_empty() {
            assert!(
                !content.contains("<<"),
                "Found unrendered minijinja opening delimiter in {:?}",
                entry.path()
            );
            assert!(
                !content.contains(">>"),
                "Found unrendered minijinja closing delimiter in {:?}",
                entry.path()
            );
        }
    }
}
