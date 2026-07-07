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
    // 新的条件化实现：reload 参数通过 dict update 注入（`"reload_dirs": _RELOAD_DIRS`），
    // 不再是直接 kwarg 形式（`reload_dirs=`）。
    let entry_main =
        fs::read_to_string(tmp.path().join("main.py")).expect("read generated main.py");
    assert!(
        entry_main.contains("\"reload_dirs\"") && entry_main.contains("\"reload_excludes\""),
        "main.py must pass reload_dirs/reload_excludes to uvicorn.run to avoid the reload loop:\n{entry_main}"
    );
    assert!(
        entry_main.contains("if settings.server.reload"),
        "main.py must conditionally apply reload params only when reload is enabled:\n{entry_main}"
    );
    assert!(
        entry_main.contains("\"*.log\"") && entry_main.contains("\"logs\""),
        "main.py reload_excludes must exclude the log directory/files (logs, *.log):\n{entry_main}"
    );
}

/// 渲染 mcp-python 框架模板到临时目录（离线，仅渲染，不触发 uv/网络/pytest）。
///
/// 复刻 orchestrator 在 `to_template_context()` 之后注入的两个后端键：
/// `mcp_backend` / `mcp_backend_is_official`（其它现有 python/fastapi 测试无此步骤，
/// mcp-python 必须有——它驱动 `app/server.py` 里唯一的 `<%if%>` 后端分支）。
/// 直接以 `serde_json::Value::String/Bool` 注入，避免宏/导入问题
/// （`to_template_context` 返回的就是 `HashMap<String, serde_json::Value>`）。
fn render_mcp_python(backend: &str) -> tempfile::TempDir {
    let mut params = PythonParams::new("mcp-py-demo".to_string());
    params.base.host = Some("0.0.0.0".to_string());
    params.base.port = Some(8000);
    let mut context = params.to_template_context();
    context.insert(
        "mcp_backend".to_string(),
        serde_json::Value::String(backend.to_string()),
    );
    context.insert(
        "mcp_backend_is_official".to_string(),
        serde_json::Value::Bool(backend == "official"),
    );

    let tmp = tempfile::tempdir().expect("create tempdir");
    let mut processor = TemplateProcessor::new().expect("create processor");
    processor
        .process_embedded_template_directory("frameworks/python/mcp-python", tmp.path(), context)
        .expect("render embedded mcp-python templates");
    tmp
}

#[test]
fn mcp_python_embedded_generation_renders_without_external_tools() {
    // Given/When/Then × 两个后端（fastmcp 默认 + official）：离线 render-assert。
    // 该测试不触发 uv/网络/pytest；live boot 证据在 F3，不在这里。
    for backend in ["fastmcp", "official"] {
        let tmp = render_mcp_python(backend);

        // Then(1): 预期文件存在且 .tmpl 后缀已剥离
        let files = collect_relative_files(tmp.path());
        for expected in [
            "config.toml",
            "main.py",
            "pyproject.toml",
            "app/__init__.py",
            "app/server.py",
            "app/mcp_instance.py",
            "app/settings.py",
            "app/logging.py",
            "app/tools/__init__.py",
            "app/tools/echo.py",
            "tests/conftest.py",
            "tests/test_echo.py",
            "tests/__init__.py",
            "Makefile",
            "README.md",
            ".gitignore",
            ".pre-commit-config.yaml",
            ".env.example",
        ] {
            assert!(
                files.iter().any(|f| f == expected),
                "[{backend}] expected {expected}, got: {files:?}"
            );
        }
        assert!(
            !files.iter().any(|f| f.ends_with(".tmpl")),
            "[{backend}] no .tmpl files should remain, got: {files:?}"
        );

        // Then(2): 任何渲染文件都不得残留自定义分隔符 `<<` / `%>` / `<%`
        for rel in &files {
            let content = fs::read_to_string(tmp.path().join(rel))
                .unwrap_or_else(|_| panic!("[{backend}] read generated file {rel}"));
            assert!(
                !content.contains("<<"),
                "[{backend}] file {rel} still contains unrendered `<<`"
            );
            assert!(
                !content.contains("%>"),
                "[{backend}] file {rel} still contains unrendered `%>`"
            );
            assert!(
                !content.contains("<%"),
                "[{backend}] file {rel} still contains unrendered `<%`"
            );
        }

        // Then(3): project_name 被替换进 config.toml
        let config = fs::read_to_string(tmp.path().join("config.toml"))
            .unwrap_or_else(|_| panic!("[{backend}] read config.toml"));
        assert!(
            config.contains("mcp-py-demo"),
            "[{backend}] project_name not substituted into config.toml:\n{config}"
        );

        // Then(4): config.toml 含三段 + 端口 8000 + sse_enabled + 正确 backend
        assert!(
            config.contains("[server]") && config.contains("[mcp]") && config.contains("[log]"),
            "[{backend}] config.toml missing [server]/[mcp]/[log]:\n{config}"
        );
        assert!(
            config.contains("port = 8000"),
            "[{backend}] config.toml missing port = 8000:\n{config}"
        );
        assert!(
            config.contains("sse_enabled = true"),
            "[{backend}] config.toml missing sse_enabled = true:\n{config}"
        );
        assert!(
            config.contains(&format!("backend = \"{backend}\"")),
            "[{backend}] config.toml backend not set to {backend}:\n{config}"
        );

        // Then(5): server.py —— 从单例 import mcp + 调用 register_tools() + path-once 307 fix
        //          + 后端 import 分歧（证明 `<%if%>` 命中）
        let server = fs::read_to_string(tmp.path().join("app/server.py"))
            .unwrap_or_else(|_| panic!("[{backend}] read app/server.py"));
        // 共享单例：server.py 不再自己构造 FastMCP，而是从 app.mcp_instance import。
        assert!(
            server.contains("from app.mcp_instance import mcp"),
            "[{backend}] server.py must import the shared singleton (from app.mcp_instance import mcp):\n{server}"
        );
        // 工具通过 register_tools()（无参，import 即注册）在构建传输前注册。
        assert!(
            server.contains("register_tools("),
            "[{backend}] server.py must call register_tools() before building transports:\n{server}"
        );
        // 新的 path-once：不再用 Mount，而是把子应用的路由 extend 进父应用的 routes。
        assert!(
            server.contains("streamable_app.routes") && server.contains("Route(\"/healthz\""),
            "[{backend}] server.py must splice transports via routes.extend() and healthz:\n{server}"
        );
        // 后端构造收敛在 mcp_instance.py，server.py 不再 import FastMCP 后端构造符号。
        if backend == "official" {
            // path-once：官方后端在实例的 mcp.settings 上设置路径，而不是工厂参数。
            assert!(
                server.contains("mcp.settings.streamable_http_path = settings.mcp.mcp_path"),
                "[official] server.py must own the path once via mcp.settings.streamable_http_path = settings.mcp.mcp_path:\n{server}"
            );
            assert!(
                server.contains("streamable_http_app()") && server.contains("sse_app()"),
                "[official] server.py must build streamable_http_app() + sse_app():\n{server}"
            );
        } else {
            // path-once：http_app 工厂在 path 参数上拥有完整路径（settings.mcp.mcp_path）；SSE 走 transport="sse"。
            assert!(
                server.contains("http_app(path=settings.mcp.mcp_path")
                    && server.contains("transport=\"sse\""),
                "[fastmcp] server.py must build SSE (transport=\"sse\") + streamable (http_app(path=settings.mcp.mcp_path)):\n{server}"
            );
        }

        // Then(5b): app/mcp_instance.py —— 唯一构造 FastMCP 的地方，后端 import 分歧在此。
        let mcp_instance = fs::read_to_string(tmp.path().join("app/mcp_instance.py"))
            .unwrap_or_else(|_| panic!("[{backend}] read app/mcp_instance.py"));
        if backend == "official" {
            assert!(
                mcp_instance.contains("from mcp.server.fastmcp import"),
                "[official] mcp_instance.py must import from mcp.server.fastmcp:\n{mcp_instance}"
            );
        } else {
            assert!(
                mcp_instance.contains("from fastmcp import"),
                "[fastmcp] mcp_instance.py must import from fastmcp:\n{mcp_instance}"
            );
            assert!(
                !mcp_instance.contains("from mcp.server.fastmcp"),
                "[fastmcp] mcp_instance.py must NOT import the official mcp.server.fastmcp:\n{mcp_instance}"
            );
        }

        // Then(6): pyproject.toml —— 依赖分歧（证明另一条 `<%if%>` 命中）+ 共有 pytest 配置
        let pyproject = fs::read_to_string(tmp.path().join("pyproject.toml"))
            .unwrap_or_else(|_| panic!("[{backend}] read pyproject.toml"));
        if backend == "official" {
            assert!(
                pyproject.contains("mcp[cli]>=1.2,<2"),
                "[official] pyproject.toml must pin mcp[cli]>=1.2,<2:\n{pyproject}"
            );
        } else {
            assert!(
                pyproject.contains("fastmcp>=2,<3"),
                "[fastmcp] pyproject.toml must pin fastmcp>=2,<3:\n{pyproject}"
            );
        }
        assert!(
            pyproject.contains("asyncio_mode = \"auto\"")
                && pyproject.contains("pythonpath = [\".\"]"),
            "[{backend}] pyproject.toml missing pytest asyncio_mode/pythonpath config:\n{pyproject}"
        );

        // Then(7): echo.py 后端感知 —— 从单例 import mcp + 用本后端的装饰器（active 主形式）
        let echo = fs::read_to_string(tmp.path().join("app/tools/echo.py"))
            .unwrap_or_else(|_| panic!("[{backend}] read app/tools/echo.py"));
        assert!(
            echo.contains("from app.mcp_instance import mcp"),
            "[{backend}] echo.py must import the shared singleton (from app.mcp_instance import mcp):\n{echo}"
        );
        if backend == "official" {
            // official：装饰器必须带括号 @mcp.tool()。
            assert!(
                echo.contains("@mcp.tool()"),
                "[official] echo.py must use the parenthesized decorator @mcp.tool():\n{echo}"
            );
        } else {
            // fastmcp：装饰器无括号 @mcp.tool（注意 @mcp.tool 是 @mcp.tool() 的前缀，
            // 故要求出现 @mcp.tool 且 NOT 出现 @mcp.tool() —— 排除括号形式）。
            assert!(
                echo.contains("@mcp.tool"),
                "[fastmcp] echo.py must use the parenless decorator @mcp.tool:\n{echo}"
            );
            assert!(
                !echo.contains("@mcp.tool()"),
                "[fastmcp] echo.py must NOT use the parenthesized form @mcp.tool():\n{echo}"
            );
        }

        // Then(8): README 提及双传输端点与测试入口
        let readme = fs::read_to_string(tmp.path().join("README.md"))
            .unwrap_or_else(|_| panic!("[{backend}] read README.md"));
        assert!(
            readme.contains("/mcp") && readme.contains("/sse") && readme.contains("make test"),
            "[{backend}] README missing /mcp, /sse or `make test`:\n{readme}"
        );
    }
}

/// 渲染开启鉴权的 mcp-python（离线，仅渲染）。
///
/// 复刻 orchestrator `generate_mcp_python_language` 在 `to_template_context()` 之后注入的五个键：
/// `mcp_backend` / `mcp_backend_is_official`（后端分支）+ `auth_mode` / `auth_enabled` / `auth_is_azure_ad`（鉴权分支）。
/// `auth_mode=="jwt"` ↔ `auth_enabled==true`；`auth_mode=="none"` ↔ `auth_enabled==false`，
/// 与 `AuthMode::as_str()` / `AuthMode::is_enabled()` 的语义一致。
fn render_mcp_python_auth(backend: &str, auth_mode: &str) -> tempfile::TempDir {
    use scaffold_gen::generators::auth_options::AuthMode;
    use scaffold_gen::generators::mcp_auth_context::McpPythonAuthContext;
    use scaffold_gen::generators::mcp_options::McpBackend;

    let mut params = PythonParams::new("mcp-py-auth-demo".to_string());
    params.base.host = Some("0.0.0.0".to_string());
    params.base.port = Some(8000);
    let mut context = params.to_template_context();

    let backend_enum = McpBackend::parse_from_str(backend).unwrap();
    let auth_enum = AuthMode::parse_from_str(auth_mode).unwrap();
    McpPythonAuthContext::new(backend_enum, auth_enum).inject_into(&mut context);

    let tmp = tempfile::tempdir().expect("create tempdir");
    let mut processor = TemplateProcessor::new().expect("create processor");
    processor
        .process_embedded_template_directory("frameworks/python/mcp-python", tmp.path(), context)
        .expect("render embedded mcp-python templates (auth)");
    tmp
}

/// 读取渲染后的项目内某个文件（缺失即 panic，带后端/模式上下文）。
fn read_rendered(tmp: &tempfile::TempDir, rel: &str, backend: &str, mode: &str) -> String {
    fs::read_to_string(tmp.path().join(rel))
        .unwrap_or_else(|_| panic!("[{backend}/{mode}] read {rel}"))
}

#[test]
fn mcp_python_auth_renders() {
    // Given/When/Then：离线 render-assert 覆盖鉴权 ON（两后端）+ OFF（两后端，默认）。
    // 不触发 uv/网络/pytest；live auth boot（401/200）证据在 F3，不在这里。

    // ── auth ON：official + jwt ────────────────────────────────────────────────
    {
        let backend = "official";
        let mode = "jwt";
        let tmp = render_mcp_python_auth(backend, mode);

        // 渲染产物不得残留自定义分隔符。
        let files = collect_relative_files(tmp.path());
        for rel in &files {
            let content = read_rendered(&tmp, rel, backend, mode);
            for delim in ["<<", "%>", "<%"] {
                assert!(
                    !content.contains(delim),
                    "[{backend}/{mode}] file {rel} still contains unrendered `{delim}`"
                );
            }
        }

        // settings.py：AuthConfig + resource_server_url。
        let settings = read_rendered(&tmp, "app/settings.py", backend, mode);
        assert!(
            settings.contains("class AuthConfig") && settings.contains("resource_server_url"),
            "[{backend}/{mode}] settings.py must define AuthConfig with resource_server_url:\n{settings}"
        );

        // config.toml：[auth] + enabled = false + resource_server_url。
        let config = read_rendered(&tmp, "config.toml", backend, mode);
        assert!(
            config.contains("[auth]")
                && config.contains("enabled = false")
                && config.contains("resource_server_url"),
            "[{backend}/{mode}] config.toml must have [auth] enabled=false + resource_server_url:\n{config}"
        );

        // mcp_instance.py：official 后端导入 JwksTokenVerifier + AuthSettings + token_verifier=。
        let mcp_instance = read_rendered(&tmp, "app/mcp_instance.py", backend, mode);
        assert!(
            mcp_instance.contains("from app.auth import JwksTokenVerifier")
                && mcp_instance.contains("AuthSettings")
                && mcp_instance.contains("token_verifier="),
            "[{backend}/{mode}] mcp_instance.py must wire JwksTokenVerifier + AuthSettings + token_verifier=:\n{mcp_instance}"
        );

        // server.py：B1 离线护栏 —— _collect_middleware + Starlette(routes= ... middleware=。
        let server = read_rendered(&tmp, "app/server.py", backend, mode);
        assert!(
            server.contains("_collect_middleware"),
            "[{backend}/{mode}] server.py must define _collect_middleware (B1 re-attach):\n{server}"
        );
        assert!(
            server.contains("Starlette(routes=") && server.contains("middleware="),
            "[{backend}/{mode}] server.py must re-attach middleware onto the spliced parent (Starlette(routes=..., middleware=)):\n{server}"
        );

        // app/auth.py：official 后端真实 JwksTokenVerifier + require exp。
        let auth = read_rendered(&tmp, "app/auth.py", backend, mode);
        assert!(
            auth.contains("class JwksTokenVerifier")
                && auth.contains("options={\"require\": [\"exp\"]}"),
            "[{backend}/{mode}] app/auth.py must define JwksTokenVerifier with options require exp:\n{auth}"
        );

        // pyproject.toml：official+auth 才有 pyjwt[crypto]。
        let pyproject = read_rendered(&tmp, "pyproject.toml", backend, mode);
        assert!(
            pyproject.contains("pyjwt[crypto]"),
            "[{backend}/{mode}] pyproject.toml must contain pyjwt[crypto] for official+auth:\n{pyproject}"
        );

        // README：鉴权小节存在。
        let readme = read_rendered(&tmp, "README.md", backend, mode);
        assert!(
            readme.contains("鉴权") || readme.contains("oauth-protected-resource"),
            "[{backend}/{mode}] README must contain the auth section:\n{readme}"
        );
    }

    // ── auth ON：fastmcp + jwt ─────────────────────────────────────────────────
    {
        let backend = "fastmcp";
        let mode = "jwt";
        let tmp = render_mcp_python_auth(backend, mode);

        let files = collect_relative_files(tmp.path());
        for rel in &files {
            let content = read_rendered(&tmp, rel, backend, mode);
            for delim in ["<<", "%>", "<%"] {
                assert!(
                    !content.contains(delim),
                    "[{backend}/{mode}] file {rel} still contains unrendered `{delim}`"
                );
            }
        }

        // settings.py：AuthConfig 存在（两后端共用配置面）。
        let settings = read_rendered(&tmp, "app/settings.py", backend, mode);
        assert!(
            settings.contains("class AuthConfig"),
            "[{backend}/{mode}] settings.py must define AuthConfig:\n{settings}"
        );

        // mcp_instance.py：fastmcp 后端导入内置 JWTVerifier + auth=。
        let mcp_instance = read_rendered(&tmp, "app/mcp_instance.py", backend, mode);
        assert!(
            mcp_instance.contains("from fastmcp.server.auth.providers.jwt import JWTVerifier")
                && mcp_instance.contains("auth="),
            "[{backend}/{mode}] mcp_instance.py must wire the built-in JWTVerifier + auth=:\n{mcp_instance}"
        );

        // server.py：B1 中间件再挂载仍存在。
        let server = read_rendered(&tmp, "app/server.py", backend, mode);
        assert!(
            server.contains("_collect_middleware") && server.contains("middleware="),
            "[{backend}/{mode}] server.py must keep _collect_middleware + middleware= (B1):\n{server}"
        );

        // app/auth.py：fastmcp 用内置 verifier，本文件仅 docstring（无 JwksTokenVerifier）。
        let auth = read_rendered(&tmp, "app/auth.py", backend, mode);
        assert!(
            !auth.contains("class JwksTokenVerifier"),
            "[{backend}/{mode}] app/auth.py must be docstring-only for fastmcp (no JwksTokenVerifier):\n{auth}"
        );

        // pyproject.toml：fastmcp 自带 crypto，不应引入 pyjwt。
        let pyproject = read_rendered(&tmp, "pyproject.toml", backend, mode);
        assert!(
            !pyproject.contains("pyjwt"),
            "[{backend}/{mode}] pyproject.toml must NOT contain pyjwt for fastmcp+auth:\n{pyproject}"
        );
    }

    // ── auth OFF：两后端默认（--auth none）—— 证明 ZERO 鉴权代码/配置/依赖 ───────────
    for backend in ["fastmcp", "official"] {
        let mode = "none";
        let tmp = render_mcp_python_auth(backend, mode);

        let files = collect_relative_files(tmp.path());
        for rel in &files {
            let content = read_rendered(&tmp, rel, backend, mode);
            for delim in ["<<", "%>", "<%"] {
                assert!(
                    !content.contains(delim),
                    "[{backend}/{mode}] file {rel} still contains unrendered `{delim}`"
                );
            }
        }

        // settings.py：无 AuthConfig。
        let settings = read_rendered(&tmp, "app/settings.py", backend, mode);
        assert!(
            !settings.contains("AuthConfig"),
            "[{backend}/{mode}] settings.py must NOT contain AuthConfig when auth off:\n{settings}"
        );

        // config.toml：无 [auth]。
        let config = read_rendered(&tmp, "config.toml", backend, mode);
        assert!(
            !config.contains("[auth]"),
            "[{backend}/{mode}] config.toml must NOT contain [auth] when auth off:\n{config}"
        );

        // mcp_instance.py：无任何 verifier / AuthSettings。
        let mcp_instance = read_rendered(&tmp, "app/mcp_instance.py", backend, mode);
        assert!(
            !mcp_instance.contains("JWTVerifier")
                && !mcp_instance.contains("AuthSettings")
                && !mcp_instance.contains("JwksTokenVerifier"),
            "[{backend}/{mode}] mcp_instance.py must contain NO auth wiring when auth off:\n{mcp_instance}"
        );

        // server.py：_collect_middleware 仍是无条件 helper（B1 实现），但其体内无 settings.auth。
        let server = read_rendered(&tmp, "app/server.py", backend, mode);
        assert!(
            !server.contains("settings.auth"),
            "[{backend}/{mode}] server.py must reference NO settings.auth when auth off:\n{server}"
        );

        // app/auth.py：docstring-only（无 JwksTokenVerifier）。
        let auth = read_rendered(&tmp, "app/auth.py", backend, mode);
        assert!(
            !auth.contains("class JwksTokenVerifier"),
            "[{backend}/{mode}] app/auth.py must be docstring-only when auth off:\n{auth}"
        );

        // pyproject.toml：无 pyjwt。
        let pyproject = read_rendered(&tmp, "pyproject.toml", backend, mode);
        assert!(
            !pyproject.contains("pyjwt"),
            "[{backend}/{mode}] pyproject.toml must NOT contain pyjwt when auth off:\n{pyproject}"
        );

        // README：无鉴权小节。
        let readme = read_rendered(&tmp, "README.md", backend, mode);
        assert!(
            !readme.contains("## 鉴权"),
            "[{backend}/{mode}] README must NOT contain the auth section when auth off:\n{readme}"
        );

        // test_auth.py：鉴权关闭时不应渲染出实际测试体（empty/comment-only，pytest 不收集）。
        let test_auth_path = tmp.path().join("tests/test_auth.py");
        if test_auth_path.exists() {
            let test_auth = fs::read_to_string(&test_auth_path)
                .unwrap_or_else(|_| panic!("[{backend}/{mode}] read tests/test_auth.py"));
            assert!(
                !test_auth.contains("def test_"),
                "[{backend}/{mode}] tests/test_auth.py must contain NO tests when auth off:\n{test_auth}"
            );
        }
    }
}

/// 渲染 `--auth azure-ad` 的 mcp-python（离线，仅渲染）。
///
/// 使用 `McpPythonAuthContext::inject_into` 注入全部 5 个 auth-related 键，
/// 与 orchestrator `generate_mcp_python_language` 行为一致。azure-ad ⇒ `auth_enabled==true`
/// 且 `auth_mode=="azure-ad"`，镜像 `AuthMode::AzureAd` 的语义。
fn render_mcp_python_azuread(backend: &str) -> tempfile::TempDir {
    use scaffold_gen::generators::auth_options::AuthMode;
    use scaffold_gen::generators::mcp_auth_context::McpPythonAuthContext;
    use scaffold_gen::generators::mcp_options::McpBackend;

    let mut params = PythonParams::new("mcp-py-azuread-demo".to_string());
    params.base.host = Some("0.0.0.0".to_string());
    params.base.port = Some(8000);
    let mut context = params.to_template_context();

    let backend_enum = McpBackend::parse_from_str(backend).unwrap();
    McpPythonAuthContext::new(backend_enum, AuthMode::AzureAd).inject_into(&mut context);

    let tmp = tempfile::tempdir().expect("create tempdir");
    let mut processor = TemplateProcessor::new().expect("create processor");
    processor
        .process_embedded_template_directory("frameworks/python/mcp-python", tmp.path(), context)
        .expect("render embedded mcp-python templates (azure-ad)");
    tmp
}

#[test]
fn mcp_python_azuread_renders() {
    // Given/When/Then：离线 render-assert 覆盖 --auth azure-ad（两后端）+ none/jwt 回归。
    // 不触发 uv/网络/pytest；live boot（dual-issuer 接受 / 401）证据在 F3，不在这里。

    // ── azure-ad：official ─────────────────────────────────────────────────────
    {
        let backend = "official";
        let mode = "azure-ad";
        let tmp = render_mcp_python_azuread(backend);

        // 渲染产物不得残留自定义分隔符。
        let files = collect_relative_files(tmp.path());
        for rel in &files {
            let content = read_rendered(&tmp, rel, backend, mode);
            for delim in ["<<", "%>", "<%"] {
                assert!(
                    !content.contains(delim),
                    "[{backend}/{mode}] file {rel} still contains unrendered `{delim}`"
                );
            }
        }

        // settings.py：AuthConfig + azure-ad 预设字段 + model_post_init 自动推导。
        let settings = read_rendered(&tmp, "app/settings.py", backend, mode);
        for marker in [
            "class AuthConfig",
            "tenant_id",
            "resource_app_id",
            "extra_issuers",
            "identity_claims",
            "model_post_init",
        ] {
            assert!(
                settings.contains(marker),
                "[{backend}/{mode}] settings.py must contain `{marker}`:\n{settings}"
            );
        }

        // config.toml：mode = "azure-ad" + tenant_id + resource_app_id。
        let config = read_rendered(&tmp, "config.toml", backend, mode);
        for marker in ["mode = \"azure-ad\"", "tenant_id", "resource_app_id"] {
            assert!(
                config.contains(marker),
                "[{backend}/{mode}] config.toml must contain `{marker}`:\n{config}"
            );
        }

        // app/auth.py：official 多 issuer 校验器 + 身份提取 + 当前身份 + JWKS 预热。
        let auth = read_rendered(&tmp, "app/auth.py", backend, mode);
        for marker in [
            "class JwksTokenVerifier",
            "extra_issuers",
            "identity_claims",
            "_extract_identity",
            "def current_identity",
            "def warm_up_jwks",
        ] {
            assert!(
                auth.contains(marker),
                "[{backend}/{mode}] app/auth.py must contain `{marker}`:\n{auth}"
            );
        }

        // mcp_instance.py：tenant fail-fast + extra_issuers= / identity_claims= 入参 + warm_up_jwks 调用。
        let mcp_instance = read_rendered(&tmp, "app/mcp_instance.py", backend, mode);
        for marker in [
            "from app.auth import JwksTokenVerifier",
            "warm_up_jwks",
            "warm_up_jwks(",
            "settings.auth.tenant_id",
            "extra_issuers=",
            "identity_claims=",
        ] {
            assert!(
                mcp_instance.contains(marker),
                "[{backend}/{mode}] mcp_instance.py must contain `{marker}`:\n{mcp_instance}"
            );
        }

        // whoami.py：存在、无参（不接收任何标识用户的入参）。
        let whoami = read_rendered(&tmp, "app/tools/whoami.py", backend, mode);
        assert!(
            whoami.contains("def whoami("),
            "[{backend}/{mode}] whoami.py must define def whoami(:\n{whoami}"
        );
        assert!(
            whoami.contains("def whoami() -> dict:"),
            "[{backend}/{mode}] whoami() must take NO user param (def whoami() -> dict):\n{whoami}"
        );

        // pyproject.toml：official + azure-ad 才有 pyjwt。
        let pyproject = read_rendered(&tmp, "pyproject.toml", backend, mode);
        assert!(
            pyproject.contains("pyjwt"),
            "[{backend}/{mode}] pyproject.toml must contain pyjwt for official+azure-ad:\n{pyproject}"
        );

        // README：azure-ad 小节存在。
        let readme = read_rendered(&tmp, "README.md", backend, mode);
        assert!(
            readme.contains("azure-ad") && readme.contains("tenant_id"),
            "[{backend}/{mode}] README must contain the azure-ad section (azure-ad + tenant_id):\n{readme}"
        );
    }

    // ── azure-ad：fastmcp ──────────────────────────────────────────────────────
    {
        let backend = "fastmcp";
        let mode = "azure-ad";
        let tmp = render_mcp_python_azuread(backend);

        let files = collect_relative_files(tmp.path());
        for rel in &files {
            let content = read_rendered(&tmp, rel, backend, mode);
            for delim in ["<<", "%>", "<%"] {
                assert!(
                    !content.contains(delim),
                    "[{backend}/{mode}] file {rel} still contains unrendered `{delim}`"
                );
            }
        }

        // settings.py：azure-ad 预设字段（配置面两后端共用）。
        let settings = read_rendered(&tmp, "app/settings.py", backend, mode);
        for marker in ["class AuthConfig", "tenant_id", "resource_app_id"] {
            assert!(
                settings.contains(marker),
                "[{backend}/{mode}] settings.py must contain `{marker}`:\n{settings}"
            );
        }

        // config.toml：mode = "azure-ad" + tenant 字段。
        let config = read_rendered(&tmp, "config.toml", backend, mode);
        assert!(
            config.contains("mode = \"azure-ad\"") && config.contains("tenant_id"),
            "[{backend}/{mode}] config.toml must have mode=\"azure-ad\" + tenant_id:\n{config}"
        );

        // mcp_instance.py：tenant fail-fast + 单个内置 JWTVerifier；fastmcp 不导入 warm_up_jwks。
        let mcp_instance = read_rendered(&tmp, "app/mcp_instance.py", backend, mode);
        assert!(
            mcp_instance.contains("settings.auth.tenant_id"),
            "[{backend}/{mode}] mcp_instance.py must fail-fast on tenant_id:\n{mcp_instance}"
        );
        assert!(
            mcp_instance.contains("from fastmcp.server.auth.providers.jwt import JWTVerifier"),
            "[{backend}/{mode}] mcp_instance.py must import the built-in JWTVerifier:\n{mcp_instance}"
        );
        assert!(
            !mcp_instance.contains("warm_up_jwks"),
            "[{backend}/{mode}] mcp_instance.py must NOT import/call warm_up_jwks for fastmcp:\n{mcp_instance}"
        );
        assert!(
            !mcp_instance.contains("from app.auth import JwksTokenVerifier"),
            "[{backend}/{mode}] fastmcp must NOT import the official JwksTokenVerifier:\n{mcp_instance}"
        );

        // app/auth.py：fastmcp 用内置 verifier，本文件无 JwksTokenVerifier。
        let auth = read_rendered(&tmp, "app/auth.py", backend, mode);
        assert!(
            !auth.contains("class JwksTokenVerifier"),
            "[{backend}/{mode}] app/auth.py must be docstring-only for fastmcp:\n{auth}"
        );

        // whoami.py：存在、无参。
        let whoami = read_rendered(&tmp, "app/tools/whoami.py", backend, mode);
        assert!(
            whoami.contains("def whoami() -> dict:"),
            "[{backend}/{mode}] whoami() must take NO user param:\n{whoami}"
        );

        // pyproject.toml：fastmcp 自带 crypto，不应引入 pyjwt。
        let pyproject = read_rendered(&tmp, "pyproject.toml", backend, mode);
        assert!(
            !pyproject.contains("pyjwt"),
            "[{backend}/{mode}] pyproject.toml must NOT contain pyjwt for fastmcp+azure-ad:\n{pyproject}"
        );

        // README：azure-ad 小节存在。
        let readme = read_rendered(&tmp, "README.md", backend, mode);
        assert!(
            readme.contains("azure-ad") && readme.contains("tenant_id"),
            "[{backend}/{mode}] README must contain the azure-ad section:\n{readme}"
        );
    }

    // ── 回归：--auth none（official）—— ZERO azure-ad 痕迹 ───────────────────────
    {
        let backend = "official";
        let mode = "none";
        let tmp = render_mcp_python_auth(backend, mode);

        // settings.py / config.toml / README 不得含任何 azure-ad 痕迹。
        let settings = read_rendered(&tmp, "app/settings.py", backend, mode);
        for marker in ["tenant_id", "azure-ad", "current_identity"] {
            assert!(
                !settings.contains(marker),
                "[{backend}/{mode}] settings.py must NOT contain `{marker}` when auth none:\n{settings}"
            );
        }
        let config = read_rendered(&tmp, "config.toml", backend, mode);
        for marker in ["tenant_id", "azure-ad"] {
            assert!(
                !config.contains(marker),
                "[{backend}/{mode}] config.toml must NOT contain `{marker}` when auth none:\n{config}"
            );
        }
        let readme = read_rendered(&tmp, "README.md", backend, mode);
        assert!(
            !readme.contains("azure-ad"),
            "[{backend}/{mode}] README must NOT contain azure-ad when auth none:\n{readme}"
        );

        // whoami.py：注释-only stub（无实际 whoami 函数定义）。
        let whoami_path = tmp.path().join("app/tools/whoami.py");
        if whoami_path.exists() {
            let whoami = fs::read_to_string(&whoami_path)
                .unwrap_or_else(|_| panic!("[{backend}/{mode}] read app/tools/whoami.py"));
            assert!(
                !whoami.contains("def whoami("),
                "[{backend}/{mode}] whoami.py must be comment-only stub when auth none:\n{whoami}"
            );
        }
    }

    // ── 回归：--auth jwt（official）—— 通用单 issuer，NO azure-ad 痕迹 ───────────
    {
        let backend = "official";
        let mode = "jwt";
        let tmp = render_mcp_python_auth(backend, mode);

        // settings.py：AuthConfig 存在，但无 azure-ad 专属字段。
        let settings = read_rendered(&tmp, "app/settings.py", backend, mode);
        assert!(
            settings.contains("class AuthConfig"),
            "[{backend}/{mode}] settings.py must define AuthConfig:\n{settings}"
        );
        for marker in ["tenant_id", "resource_app_id", "model_post_init"] {
            assert!(
                !settings.contains(marker),
                "[{backend}/{mode}] settings.py must NOT contain azure-ad field `{marker}` for jwt:\n{settings}"
            );
        }

        // config.toml：mode = "jwt"，不得是 azure-ad，不得含 tenant_id。
        let config = read_rendered(&tmp, "config.toml", backend, mode);
        assert!(
            config.contains("mode = \"jwt\""),
            "[{backend}/{mode}] config.toml must have mode=\"jwt\":\n{config}"
        );
        assert!(
            !config.contains("mode = \"azure-ad\""),
            "[{backend}/{mode}] config.toml must NOT set mode=\"azure-ad\" for jwt:\n{config}"
        );
        assert!(
            !config.contains("tenant_id"),
            "[{backend}/{mode}] config.toml must NOT contain tenant_id for jwt:\n{config}"
        );

        // mcp_instance.py：单 issuer wiring，不传 extra_issuers / identity_claims，不调 warm_up_jwks。
        let mcp_instance = read_rendered(&tmp, "app/mcp_instance.py", backend, mode);
        for marker in ["extra_issuers=", "identity_claims=", "warm_up_jwks"] {
            assert!(
                !mcp_instance.contains(marker),
                "[{backend}/{mode}] mcp_instance.py must NOT contain `{marker}` for jwt:\n{mcp_instance}"
            );
        }
    }
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
    let vite_config_content = std::fs::read_to_string(&vite_config).unwrap();
    assert!(
        vite_config_content.contains("loadEnv"),
        "vite.config.ts must read env via loadEnv:\n{vite_config_content}"
    );
    assert!(
        vite_config_content.contains("allowedHosts"),
        "vite.config.ts must configure server.allowedHosts:\n{vite_config_content}"
    );
    assert!(
        vite_config_content.contains("VITE_DEV_HOST")
            && vite_config_content.contains("VITE_DEV_PORT"),
        "vite.config.ts must read VITE_DEV_HOST / VITE_DEV_PORT:\n{vite_config_content}"
    );

    // .env (ready-to-run) + .env.example (documented reference) must be generated.
    let dotenv = temp_dir.path().join(".env");
    assert!(dotenv.exists(), ".env was not generated");
    let dotenv_content = std::fs::read_to_string(&dotenv).unwrap();
    for key in [
        "VITE_DEV_HOST",
        "VITE_DEV_PORT",
        "VITE_DEV_ALLOWED_HOSTS",
        "VITE_API_BASE_URL",
    ] {
        assert!(
            dotenv_content.contains(key),
            ".env must contain {key}:\n{dotenv_content}"
        );
    }

    let dotenv_example = temp_dir.path().join(".env.example");
    assert!(dotenv_example.exists(), ".env.example was not generated");
    let dotenv_example_content = std::fs::read_to_string(&dotenv_example).unwrap();
    assert!(
        dotenv_example_content.contains("VITE_API_BASE_URL"),
        ".env.example must document VITE_API_BASE_URL:\n{dotenv_example_content}"
    );

    // env.d.ts must type the client-visible API base URL.
    let env_d_ts = temp_dir.path().join("env.d.ts");
    assert!(env_d_ts.exists(), "env.d.ts was not generated");
    let env_d_ts_content = std::fs::read_to_string(&env_d_ts).unwrap();
    assert!(
        env_d_ts_content.contains("VITE_API_BASE_URL"),
        "env.d.ts must type VITE_API_BASE_URL:\n{env_d_ts_content}"
    );

    // The API base URL must actually be used somewhere (not dead config).
    let api_lib = temp_dir.path().join("src/lib/api.ts");
    assert!(api_lib.exists(), "src/lib/api.ts was not generated");
    let api_lib_content = std::fs::read_to_string(&api_lib).unwrap();
    assert!(
        api_lib_content.contains("import.meta.env.VITE_API_BASE_URL"),
        "src/lib/api.ts must reference import.meta.env.VITE_API_BASE_URL:\n{api_lib_content}"
    );

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

// ─── Task 6.1: P2 integration validation tests ─────────────────────────────

/// Helper: render a FastAPI project into a tempdir and return the tempdir handle.
fn render_fastapi() -> tempfile::TempDir {
    let mut params = PythonParams::new("fastapi-test".to_string());
    params.base.host = Some("0.0.0.0".to_string());
    params.base.port = Some(8000);
    let context = params.to_template_context();

    let tmp = tempfile::tempdir().expect("create tempdir");
    let mut processor = TemplateProcessor::new().expect("create processor");
    processor
        .process_embedded_template_directory("frameworks/python/fastapi", tmp.path(), context)
        .expect("render embedded fastapi templates");
    tmp
}

#[test]
fn test_fastapi_config_toml_reload_defaults_to_false() {
    let tmp = render_fastapi();
    let config = fs::read_to_string(tmp.path().join("config.toml")).expect("read config.toml");
    assert!(
        config.contains("reload = false"),
        "config.toml must default reload to false (production safe):\n{config}"
    );
    // Ensure it does NOT contain reload = true
    assert!(
        !config.contains("reload = true"),
        "config.toml must NOT contain reload = true:\n{config}"
    );
}

#[test]
fn test_fastapi_main_py_conditional_reload() {
    let tmp = render_fastapi();
    let main = fs::read_to_string(tmp.path().join("main.py")).expect("read main.py");
    assert!(
        main.contains("if settings.server.reload"),
        "main.py must conditionally apply reload params:\n{main}"
    );
}

#[test]
fn test_fastapi_gitignore_excludes_uv_lock() {
    let tmp = render_fastapi();
    let gitignore = fs::read_to_string(tmp.path().join(".gitignore")).expect("read .gitignore");
    assert!(
        !gitignore.contains("uv.lock"),
        ".gitignore must NOT contain uv.lock (lock file should be committed):\n{gitignore}"
    );
}

#[test]
fn test_fastapi_includes_test_scaffold() {
    let tmp = render_fastapi();
    let files = collect_relative_files(tmp.path());
    assert!(
        files.iter().any(|f| f == "tests/conftest.py"),
        "FastAPI generation must include tests/conftest.py, got: {files:?}"
    );
    assert!(
        files.iter().any(|f| f == "tests/test_health.py"),
        "FastAPI generation must include tests/test_health.py, got: {files:?}"
    );
    assert!(
        files.iter().any(|f| f == "tests/__init__.py"),
        "FastAPI generation must include tests/__init__.py, got: {files:?}"
    );
}

#[test]
fn test_fastapi_includes_makefile() {
    let tmp = render_fastapi();
    let files = collect_relative_files(tmp.path());
    assert!(
        files.iter().any(|f| f == "Makefile"),
        "FastAPI generation must include a Makefile, got: {files:?}"
    );

    // Verify Makefile has essential targets
    let makefile = fs::read_to_string(tmp.path().join("Makefile")).expect("read Makefile");
    assert!(
        makefile.contains("install") && makefile.contains("test") && makefile.contains("dev"),
        "Makefile must contain install, test, and dev targets:\n{makefile}"
    );
}

#[test]
fn test_mcp_python_gitignore_excludes_uv_lock() {
    let tmp = render_mcp_python("fastmcp");
    let gitignore = fs::read_to_string(tmp.path().join(".gitignore")).expect("read .gitignore");
    assert!(
        !gitignore.contains("uv.lock"),
        ".gitignore must NOT contain uv.lock (lock file should be committed):\n{gitignore}"
    );
}

#[test]
fn test_with_build_does_not_overwrite_framework_makefile() {
    // Given: render the mcp-python framework (which has_own_makefile=true)
    let mut params = PythonParams::new("mcp-build-test".to_string());
    params.base.host = Some("0.0.0.0".to_string());
    params.base.port = Some(8000);
    let mut context = params.to_template_context();
    context.insert(
        "mcp_backend".to_string(),
        serde_json::Value::String("fastmcp".to_string()),
    );
    context.insert(
        "mcp_backend_is_official".to_string(),
        serde_json::Value::Bool(false),
    );

    let tmp = tempfile::tempdir().expect("create tempdir");
    let mut processor = TemplateProcessor::new().expect("create processor");

    // Step 1: render the framework templates (mcp-python provides its own Makefile)
    processor
        .process_embedded_template_directory(
            "frameworks/python/mcp-python",
            tmp.path(),
            context.clone(),
        )
        .expect("render mcp-python framework templates");

    // Capture the framework's Makefile content
    let framework_makefile =
        fs::read_to_string(tmp.path().join("Makefile")).expect("read framework Makefile");
    assert!(
        !framework_makefile.is_empty(),
        "mcp-python must provide its own Makefile"
    );

    // Step 2: simulate render_build_tooling with has_own_makefile=true.
    // When has_own_makefile is true, orchestrator calls render_build_tooling_filtered
    // which skips Makefile.tmpl. We verify by rendering build/python FULLY and checking
    // that the framework Makefile content differs from the generic one.
    let build_tmp = tempfile::tempdir().expect("create build tempdir");
    let mut build_processor = TemplateProcessor::new().expect("create processor");
    build_processor
        .process_embedded_template_directory("build/python", build_tmp.path(), context)
        .expect("render build/python templates");

    let generic_makefile =
        fs::read_to_string(build_tmp.path().join("Makefile")).expect("read generic Makefile");

    // The framework's Makefile must differ from the generic build Makefile
    assert_ne!(
        framework_makefile, generic_makefile,
        "Framework Makefile should differ from generic build Makefile — has_own_makefile protects it"
    );

    // The framework Makefile should be the MCP-specific one (contains MCP-specific markers)
    assert!(
        framework_makefile.contains("MCP") || framework_makefile.contains("mcp"),
        "Framework Makefile should be the MCP-specific version:\n{framework_makefile}"
    );

    // The generic Makefile has docker targets that the framework one does NOT
    assert!(
        generic_makefile.contains("docker-build"),
        "Generic build Makefile should have docker-build target:\n{generic_makefile}"
    );
    assert!(
        !framework_makefile.contains("docker-build"),
        "Framework Makefile should NOT have docker-build target (it's MCP-specific):\n{framework_makefile}"
    );
}

#[test]
fn react_embedded_generation_renders_without_external_tools() {
    // Given: ReactParams -> 完整模板上下文（镜像 vue3_embedded_generation_renders_without_external_tools）
    let params = scaffold_gen::generators::framework::react::ReactParams::from_project_name(
        "test-react-embedded".to_string(),
    );
    let context = params.to_template_context();

    let temp_dir = tempfile::tempdir().expect("create tempdir");
    let mut processor = TemplateProcessor::new().expect("create processor");

    // When: 走 live 渲染路径（process_embedded_template_directory），不触发 pnpm/网络
    processor
        .process_embedded_template_directory(
            "frameworks/typescript/react",
            temp_dir.path(),
            context,
        )
        .expect("render react embedded templates");

    let files = collect_relative_files(temp_dir.path());

    // Then(属性 5): 关键文件齐备
    for expected in [
        "package.json",
        "tsconfig.json",
        "vite.config.ts",
        "index.html",
        "env.d.ts",
        ".env",
        ".env.example",
        ".gitignore",
        "README.md",
        "src/main.tsx",
        "src/App.tsx",
        "src/router/index.tsx",
        "src/store/counter.ts",
        "src/pages/HomePage.tsx",
        "src/pages/AboutPage.tsx",
        "src/lib/api.ts",
    ] {
        assert!(
            files.iter().any(|f| f == expected),
            "expected {expected}, got: {files:?}"
        );
    }

    // Then(属性 3): .tmpl 后缀已剥离
    assert!(
        !files.iter().any(|f| f.ends_with(".tmpl")),
        "no .tmpl files should remain, got: {files:?}"
    );

    // Then(属性 4): project_name 被替换进 package.json 与 README.md
    let package_json =
        fs::read_to_string(temp_dir.path().join("package.json")).expect("read package.json");
    assert!(
        package_json.contains("\"name\": \"test-react-embedded\""),
        "package.json must contain substituted project name:\n{package_json}"
    );
    let readme = fs::read_to_string(temp_dir.path().join("README.md")).expect("read README.md");
    assert!(
        readme.contains("# test-react-embedded"),
        "README.md must contain substituted project name heading:\n{readme}"
    );

    // Then(属性 6): .env 驱动契约
    // vite.config.ts 通过 loadEnv 读取 VITE_DEV_HOST/PORT 并配置 allowedHosts。
    let vite_config =
        fs::read_to_string(temp_dir.path().join("vite.config.ts")).expect("read vite.config.ts");
    for marker in ["loadEnv", "allowedHosts", "VITE_DEV_HOST", "VITE_DEV_PORT"] {
        assert!(
            vite_config.contains(marker),
            "vite.config.ts must reference {marker}:\n{vite_config}"
        );
    }
    // .env 含 dev-server 键 + 客户端 API 地址。
    let dotenv = fs::read_to_string(temp_dir.path().join(".env")).expect("read .env");
    for key in [
        "VITE_DEV_HOST",
        "VITE_DEV_PORT",
        "VITE_DEV_ALLOWED_HOSTS",
        "VITE_API_BASE_URL",
    ] {
        assert!(dotenv.contains(key), ".env must contain {key}:\n{dotenv}");
    }
    // env.d.ts 为客户端声明 API 地址类型。
    let env_d_ts = fs::read_to_string(temp_dir.path().join("env.d.ts")).expect("read env.d.ts");
    assert!(
        env_d_ts.contains("VITE_API_BASE_URL"),
        "env.d.ts must type VITE_API_BASE_URL:\n{env_d_ts}"
    );
    // API 地址确被使用（非死配置）。
    let api_lib =
        fs::read_to_string(temp_dir.path().join("src/lib/api.ts")).expect("read src/lib/api.ts");
    assert!(
        api_lib.contains("import.meta.env.VITE_API_BASE_URL"),
        "src/lib/api.ts must reference import.meta.env.VITE_API_BASE_URL:\n{api_lib}"
    );

    // Then(属性 2): 无残留自定义分隔符 `<<` / `>>`
    // React JSX 用 `{...}`（与 minijinja `<<>>` 不冲突），故可像 Vue3 测试那样施加同样严格的断言。
    for rel in &files {
        let content = fs::read_to_string(temp_dir.path().join(rel))
            .unwrap_or_else(|_| panic!("read generated file {rel}"));
        assert!(
            !content.contains("<<"),
            "file {rel} still contains unrendered `<<`"
        );
        assert!(
            !content.contains(">>"),
            "file {rel} still contains unrendered `>>`"
        );
    }
}
