use std::fs;
use std::path::Path;

use scaffold_gen::generators::core::{Parameters, TemplateProcessor};
use scaffold_gen::generators::language::go::GoParams;
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

    // Then(4): logger 模板采用 structlog；不再残留旧 logger_state 模块
    let logger = fs::read_to_string(tmp.path().join("loggers/logger.py"))
        .expect("read generated loggers/logger.py");
    assert!(
        logger.contains("structlog"),
        "logger.py should use structlog:\n{logger}"
    );
    assert!(
        logger.contains("def init_logging"),
        "logger.py should expose init_logging:\n{logger}"
    );
    assert!(
        !files.iter().any(|f| f == "loggers/logger_state.py"),
        "logger_state.py should be removed, got: {files:?}"
    );

    // Then(5): dev 配置含 [log] 段且为 console 格式 (debug 友好)
    let dev_cfg = fs::read_to_string(tmp.path().join("config/config.dev.toml"))
        .expect("read generated config/config.dev.toml");
    assert!(
        dev_cfg.contains("[log]") && dev_cfg.contains("format = \"console\""),
        "dev config should have a [log] section in console format:\n{dev_cfg}"
    );
    // prod 配置为 json + 压缩
    let prod_cfg = fs::read_to_string(tmp.path().join("config/config.prod.toml"))
        .expect("read generated config/config.prod.toml");
    assert!(
        prod_cfg.contains("[log]")
            && prod_cfg.contains("format = \"json\"")
            && prod_cfg.contains("compress = true"),
        "prod config should have a [log] section in json format with compression:\n{prod_cfg}"
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
        "app/logging.py",
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

    // Then(4): FastAPI 也使用 structlog；config.toml 含 [log] 段；启动时初始化日志
    let app_logging = fs::read_to_string(tmp.path().join("app/logging.py"))
        .expect("read generated app/logging.py");
    assert!(
        app_logging.contains("structlog") && app_logging.contains("def init_logging"),
        "app/logging.py should use structlog and expose init_logging:\n{app_logging}"
    );
    assert!(
        config.contains("[log]"),
        "fastapi config.toml should have a [log] section:\n{config}"
    );
    let app_main =
        fs::read_to_string(tmp.path().join("app/main.py")).expect("read generated app/main.py");
    assert!(
        app_main.contains("init_logging(settings.log)"),
        "app/main.py should initialize logging at startup:\n{app_main}"
    );

    // Then(5): 入口 main.py 必须把 uvicorn 热重载限定到 app/ 并排除 logs/，
    // 否则日志写入会反复触发 reload 形成死循环 (fix/fastapi-reload-loop)。
    let entry_main =
        fs::read_to_string(tmp.path().join("main.py")).expect("read generated main.py");
    assert!(
        entry_main.contains("reload_dirs=") && entry_main.contains("reload_excludes="),
        "main.py must pass reload_dirs/reload_excludes to uvicorn.run to avoid the reload loop:\n{entry_main}"
    );
    assert!(
        entry_main.contains("\"*.log\"") && entry_main.contains("\"logs\""),
        "main.py reload_excludes must exclude the log directory/files (logs, *.log):\n{entry_main}"
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

#[test]
fn mcp_server_embedded_generation_renders_without_external_tools() {
    // Given: 一个带 host/port 的 GoParams（MCP server 复用 Go 上下文得到 module_name）
    let mut params =
        GoParams::from_project_name("mcp-demo".to_string()).with_version("1.24".to_string());
    params.base.host = Some("0.0.0.0".to_string());
    params.base.port = Some(8080);
    let context = params.to_template_context();

    let tmp = tempfile::tempdir().expect("create tempdir");
    let mut processor = TemplateProcessor::new().expect("create processor");

    // When: 走 live 渲染路径渲染 frameworks/go/mcp-server（不触发 go/buf/protoc）
    processor
        .process_embedded_template_directory("frameworks/go/mcp-server", tmp.path(), context)
        .expect("render embedded mcp-server templates");

    // Then(1): 关键文件存在且 .tmpl 后缀已剥离
    let files = collect_relative_files(tmp.path());
    for expected in [
        "go.mod",
        "cmd/server/main.go",
        "config.toml",
        "config.example.toml",
        "README.md",
        "Makefile",
        "buf.yaml",
        "buf.gen.yaml",
        "internal/config/config.go",
        "internal/mcpserver/server.go",
        "internal/transport/gin.go",
        "internal/tools/echo.go",
        "proto/echo.proto",
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

    // Then(2): 任何文件都不得残留自定义分隔符 `<<` 或 `%>`（Go 模板里不使用左移运算符）
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

    // Then(3): module_name 被替换进 go.mod，host/port 被写入 config.toml，proto 含 mcp.jsonschema 约束
    let go_mod = fs::read_to_string(tmp.path().join("go.mod")).expect("read go.mod");
    assert!(
        go_mod.contains("github.com/example/mcp-demo"),
        "module_name not substituted into go.mod:\n{go_mod}"
    );
    let config = fs::read_to_string(tmp.path().join("config.toml")).expect("read config.toml");
    assert!(
        config.contains("0.0.0.0") && config.contains("8080"),
        "host/port not driven into config.toml:\n{config}"
    );
    let proto = fs::read_to_string(tmp.path().join("proto/echo.proto")).expect("read echo.proto");
    assert!(
        proto.contains("mcp.jsonschema.required")
            && proto.contains("import \"mcp/jsonschema/jsonschema.proto\""),
        "proto missing mcp.jsonschema constraints:\n{proto}"
    );
    let readme = fs::read_to_string(tmp.path().join("README.md")).expect("read README.md");
    assert!(
        readme.contains("mcp-demo") && readme.contains("streamable") && readme.contains("/sse"),
        "README missing project name / transport sections:\n{readme}"
    );
}

#[test]
fn with_build_renders_makefile_and_dockerfile_for_python() {
    // Given: a Python project context + the build template dir (mirrors --with-build)
    let params = PythonParams::new("build-on".to_string());
    let context = params.to_template_context();
    let tmp = tempfile::tempdir().expect("create tempdir");
    let mut processor = TemplateProcessor::new().expect("create processor");

    // When: render templates/build/python (the exact path render_build_tooling uses)
    processor
        .process_embedded_template_directory("build/python", tmp.path(), context)
        .expect("render build templates");

    // Then(1): Makefile AND Dockerfile exist in the project root
    let files = collect_relative_files(tmp.path());
    assert!(
        files.iter().any(|f| f == "Makefile"),
        "expected Makefile, got: {files:?}"
    );
    assert!(
        files.iter().any(|f| f == "Dockerfile"),
        "expected Dockerfile, got: {files:?}"
    );

    // Then(2): project_name substituted, no residual delimiters
    let makefile = fs::read_to_string(tmp.path().join("Makefile")).expect("read Makefile");
    assert!(
        makefile.contains("build-on"),
        "project_name not substituted into Makefile:\n{makefile}"
    );
    for rel in &files {
        let content =
            fs::read_to_string(tmp.path().join(rel)).unwrap_or_else(|_| panic!("read {rel}"));
        assert!(
            !content.contains("<<"),
            "file {rel} still contains unrendered `<<`"
        );
        assert!(
            !content.contains("%>"),
            "file {rel} still contains unrendered `%>`"
        );
    }
}

#[test]
fn without_build_leaves_no_makefile_or_dockerfile() {
    // Given: a Python language render WITHOUT the build step
    let params = PythonParams::new("build-off".to_string());
    let context = params.to_template_context();
    let tmp = tempfile::tempdir().expect("create tempdir");
    let mut processor = TemplateProcessor::new().expect("create processor");

    // When: only the language templates are rendered (no --with-build → no build dir)
    processor
        .process_embedded_template_directory("languages/python", tmp.path(), context)
        .expect("render python templates");

    // Then: build tooling is absent
    let files = collect_relative_files(tmp.path());
    assert!(
        !files.iter().any(|f| f == "Makefile"),
        "Makefile should be absent without --with-build, got: {files:?}"
    );
    assert!(
        !files.iter().any(|f| f == "Dockerfile"),
        "Dockerfile should be absent without --with-build, got: {files:?}"
    );
}

#[test]
fn with_build_renders_makefile_and_dockerfile() {
    // Given: a Python project's template context (--with-build renders build/python)
    let mut params = PythonParams::new("build-demo".to_string());
    params.base.host = Some("0.0.0.0".to_string());
    params.base.port = Some(8000);
    let context = params.to_template_context();

    let tmp = tempfile::tempdir().expect("create tempdir");
    let mut processor = TemplateProcessor::new().expect("create processor");

    // When: rendering the unified build tooling tree into the project root
    processor
        .process_embedded_template_directory("build/python", tmp.path(), context)
        .expect("render embedded build templates");

    // Then(1): Makefile + Dockerfile exist in the output root, .tmpl stripped
    let files = collect_relative_files(tmp.path());
    assert!(
        files.iter().any(|f| f == "Makefile"),
        "expected Makefile, got: {files:?}"
    );
    assert!(
        files.iter().any(|f| f == "Dockerfile"),
        "expected Dockerfile, got: {files:?}"
    );
    assert!(
        !files.iter().any(|f| f.ends_with(".tmpl")),
        "no .tmpl files should remain, got: {files:?}"
    );

    // Then(2): no residual custom delimiters `<<` / `%>`
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

    // Then(3): project_name substituted into the rendered build tooling
    let makefile = fs::read_to_string(tmp.path().join("Makefile")).expect("read Makefile");
    assert!(
        makefile.contains("build-demo"),
        "project_name not substituted into Makefile:\n{makefile}"
    );
}

#[test]
fn without_build_flag_no_makefile_or_dockerfile() {
    // Given: a plain Python project rendered WITHOUT the build tooling tree
    let params = PythonParams::new("nobuild-demo".to_string());
    let context = params.to_template_context();

    let tmp = tempfile::tempdir().expect("create tempdir");
    let mut processor = TemplateProcessor::new().expect("create processor");

    // When: rendering only the language templates (no build/<lang>), i.e. --with-build absent
    processor
        .process_embedded_template_directory("languages/python", tmp.path(), context)
        .expect("render embedded python templates");

    // Then: build tooling is opt-in — neither Makefile nor Dockerfile is generated
    let files = collect_relative_files(tmp.path());
    assert!(
        !files.iter().any(|f| f == "Makefile"),
        "Makefile must be absent without --with-build, got: {files:?}"
    );
    assert!(
        !files.iter().any(|f| f == "Dockerfile"),
        "Dockerfile must be absent without --with-build, got: {files:?}"
    );
}
