use anyhow::{Context, Result};
use std::path::Path;

use crate::constants::{Framework, Language};
use crate::generators::auth_options::AuthMode;
use crate::generators::gin_options::GinProjectOptions;
use crate::generators::mcp_options::McpBackend;
use crate::generators::registry::{FrameworkSpec, GenKind};
use crate::generators::{
    core::{Generator, Parameters, TemplateProcessor},
    framework::gin::{GinGenerator, GinParams},
    language::go::{GoGenerator, GoParams},
    language::python::{PythonGenerator, PythonParams},
    language::rust::{RustGenerator, RustParams},
    project::{ProjectParams, ProjectScaffolder},
};
use crate::utils::env_checker::EnvironmentChecker;

/// 统一生成管线的输入 —— 由 `new.rs` 在解析出规格后填充。
///
/// 用数据 + 一个规格替代旧版 6 个签名各异的 `generate_xxx_project` 调用。
pub struct GenerationRequest<'a> {
    pub spec: FrameworkSpec,
    pub project_name: String,
    pub output_path: &'a Path,
    pub license: String,
    pub enable_precommit: bool,
    pub enable_proto_gen: bool,
    pub enable_error_gen: bool,
    pub enable_build: bool,
    pub gin_options: GinProjectOptions,
    pub mcp_backend: McpBackend,
    pub auth_mode: AuthMode,
}

/// mcp-python 生成的两个旋钮（后端 + 鉴权模式），合并为一个 typed 入参，
/// 避免 `generate_mcp_python_language` 形参过多（clippy too_many_arguments）。
struct McpPythonOptions {
    backend: McpBackend,
    auth_mode: AuthMode,
}

/// 生成器编排器，负责协调三层架构的生成器
pub struct GeneratorOrchestrator {
    pub(super) project_generator: ProjectScaffolder,
    go_generator: GoGenerator,
    python_generator: PythonGenerator,
    gin_generator: GinGenerator,
}

impl GeneratorOrchestrator {
    /// 创建新的生成器编排器
    pub fn new() -> Result<Self> {
        Ok(Self {
            project_generator: ProjectScaffolder::new()?,
            go_generator: GoGenerator::new()?,
            python_generator: PythonGenerator::new()?,
            gin_generator: GinGenerator::new()?,
        })
    }

    /// 统一的数据驱动生成入口 —— 由规格的 `kind` 选择管线。
    ///
    /// 这是替代旧版 6 个签名各异方法的唯一入口；框架差异由 `FrameworkSpec`
    /// 数据 + 一个小的 typed hook（match kind）表达，而非分散的调度。
    pub async fn generate(&mut self, mut request: GenerationRequest<'_>) -> Result<()> {
        // 规格驱动：不接受 proto/error-gen 的框架强制清零这两个开关
        if !request.spec.accepts_proto_error_gen {
            request.enable_proto_gen = false;
            request.enable_error_gen = false;
        }

        match request.spec.kind {
            GenKind::GinSync => self.generate_gin_project(
                request.project_name.clone(),
                request.output_path,
                request.gin_options.clone(),
            )?,
            GenKind::EmbeddedAsync => self.generate_embedded(&request).await?,
            GenKind::ExternalAsync => self.generate_external(&request).await?,
            GenKind::Unimplemented => return Err(anyhow::anyhow!("GoZero 项目生成尚未实现")),
        }

        if request.enable_build {
            self.render_build_tooling(&request)?;
        }

        self.format_python_project(&request);

        Ok(())
    }

    /// 尽力而为：用 ruff 格式化生成的 Python 代码。非 Python 项目或 ruff 不可用时
    /// 直接跳过（no-op）。绝不让格式化失败影响生成结果 —— 与 `init_git_repository`
    /// 的「告警继续」风格一致。
    fn format_python_project(&self, request: &GenerationRequest<'_>) {
        if request.spec.language != Language::Python {
            return;
        }
        // 优先用 PATH 上的 `ruff`；缺失时回退到 `uvx ruff`（若有 `uvx`）。
        let (program, prefix_args): (&str, &[&str]) =
            if crate::utils::toolchain::tool_available("ruff") {
                ("ruff", &[])
            } else if crate::utils::toolchain::tool_available("uvx") {
                ("uvx", &["ruff"])
            } else {
                tracing::debug!("未找到 ruff/uvx，跳过 Python 自动格式化");
                return;
            };
        let mut cmd = std::process::Command::new(program);
        cmd.args(prefix_args)
            .args(["format", "."])
            .current_dir(request.output_path);
        match cmd.status() {
            Ok(s) if s.success() => tracing::info!("已用 ruff 格式化生成的 Python 代码"),
            _ => tracing::warn!("ruff format 已跳过/失败（非致命）"),
        }
    }

    /// 项目级 `--with-build` 渲染步骤：把 `templates/build/<lang>/` 渲染进项目根目录。
    ///
    /// 在框架/语言生成之后执行，统一为所有项目类型生成配套 Makefile + Dockerfile。
    /// `<lang>` 由 `request.spec.language` 映射（见 `Language::build_dir`）；模板目录
    /// 缺失（如另一来源尚未提供 TS Dockerfile）时仅告警跳过，不阻断生成。
    fn render_build_tooling(&mut self, request: &GenerationRequest<'_>) -> Result<()> {
        let build_dir = request.spec.language.build_dir();
        let template_path = format!("build/{build_dir}");

        if !crate::template_engine::embedded_template_dir_exists(&template_path) {
            tracing::warn!(
                "构建模板目录缺失，跳过 Makefile/Dockerfile 生成: templates/{template_path}/"
            );
            return Ok(());
        }

        let context = self.build_tooling_context(request);
        let mut template_processor = TemplateProcessor::new()?;
        template_processor
            .process_embedded_template_directory(&template_path, request.output_path, context)
            .context("Failed to generate build tooling (Makefile + Dockerfile)")?;

        tracing::info!("Build tooling generated (Makefile + Dockerfile)");
        Ok(())
    }

    /// 为构建模板构造与各语言项目一致的上下文（project_name / module_name / 版本 / port 等）。
    fn build_tooling_context(
        &self,
        request: &GenerationRequest<'_>,
    ) -> std::collections::HashMap<String, serde_json::Value> {
        let project_name = request.project_name.clone();
        match request.spec.language {
            Language::Go => {
                let mut go_params = GoParams::from_project_name(project_name);
                go_params.base.host = request.gin_options.host.clone();
                go_params.base.port = request.gin_options.port;
                go_params.to_template_context()
            }
            Language::Python => {
                let mut python_params = PythonParams::new(project_name);
                python_params.base.host = request.gin_options.host.clone();
                python_params.base.port = request.gin_options.port;
                python_params.to_template_context()
            }
            Language::Rust => RustParams::new(project_name).to_template_context(),
            Language::TypeScript => ProjectParams::new(project_name).to_template_context(),
        }
    }

    /// 共享尾段：构造 ProjectParams、运行 project_generator、打印 "Next steps"。
    ///
    /// 取代旧版在 6 个方法中复制粘贴的 ProjectParams 构造 + project_generator 调用。
    fn run_project_step(
        &mut self,
        spec: &FrameworkSpec,
        project_name: &str,
        output_path: &Path,
        license: String,
        enable_precommit: bool,
    ) -> Result<()> {
        let project_params = ProjectParams::new(project_name.to_string())
            .with_license(license)
            .with_git(true)
            .with_precommit(enable_precommit)
            .with_description(spec.description(project_name));

        self.project_generator
            .generate(project_params, output_path)
            .context("Failed to generate project files")?;

        if !spec.next_steps.is_empty() {
            tracing::info!("\n📋 Next steps:");
            tracing::info!("  cd {project_name}");
            for line in spec.next_steps {
                tracing::info!("{line}");
            }
        }

        Ok(())
    }

    /// 异步内嵌模板生成（Python / 纯 Rust）。
    async fn generate_embedded(&mut self, request: &GenerationRequest<'_>) -> Result<()> {
        let project_name = &request.project_name;
        let output_path = request.output_path;

        match request.spec.language {
            Language::Python if request.spec.framework == Framework::FastApi => {
                self.generate_fastapi_language(
                    project_name.clone(),
                    output_path,
                    request.gin_options.host.clone(),
                    request.gin_options.port,
                    request.enable_precommit,
                )
                .await?;
            }
            Language::Python if request.spec.framework == Framework::McpServerPython => {
                self.generate_mcp_python_language(
                    project_name.clone(),
                    output_path,
                    request.gin_options.host.clone(),
                    request.gin_options.port,
                    request.enable_precommit,
                    McpPythonOptions {
                        backend: request.mcp_backend,
                        auth_mode: request.auth_mode,
                    },
                )
                .await?;
            }
            Language::Python => {
                self.generate_python_language(
                    project_name.clone(),
                    output_path,
                    request.enable_precommit,
                )
                .await?;
            }
            Language::Rust => {
                self.generate_rust_language(
                    project_name.clone(),
                    output_path,
                    request.enable_precommit,
                    request.enable_proto_gen,
                    request.enable_error_gen,
                )
                .await?;
            }
            Language::TypeScript if request.spec.framework == Framework::Vue3 => {
                self.generate_vue3_embedded(project_name.clone(), output_path)
                    .await?;
            }
            Language::Go if request.spec.framework == Framework::McpServer => {
                self.generate_mcp_server_embedded(
                    project_name.clone(),
                    output_path,
                    request.gin_options.host.clone(),
                    request.gin_options.port,
                )
                .await?;
            }
            Language::Go | Language::TypeScript => {
                return Err(anyhow::anyhow!(
                    "Embedded generation is not supported for {} without a framework",
                    request.spec.language
                ));
            }
        }

        self.run_project_step(
            &request.spec,
            project_name,
            output_path,
            request.license.clone(),
            request.enable_precommit,
        )?;

        tracing::info!(
            "{} project generation completed successfully!",
            request.spec.language
        );
        tracing::info!("Project created at: {}", output_path.display());

        Ok(())
    }

    /// 异步外部脚手架生成（Tauri / Vue3 / React）。
    async fn generate_external(&mut self, request: &GenerationRequest<'_>) -> Result<()> {
        match request.spec.framework {
            Framework::Tauri => self.generate_tauri_project(request).await,
            Framework::React => self.generate_react_project(request).await,
            _ => Err(anyhow::anyhow!(
                "Framework {:?} is not an external scaffolder",
                request.spec.framework
            )),
        }
    }

    /// 生成完整的Gin项目
    pub fn generate_gin_project(
        &mut self,
        project_name: String,
        output_path: &Path,
        options: GinProjectOptions,
    ) -> Result<()> {
        tracing::info!("Starting Gin project generation: {project_name}");

        let module_name = options
            .module_name
            .unwrap_or_else(|| GoParams::infer_module_name(&project_name));
        let go_version = options.go_version.unwrap_or_else(|| "1.21".to_string());
        let license = options.license.unwrap_or_else(|| "MIT".to_string());
        let description = options
            .description
            .unwrap_or_else(|| format!("A Gin web application: {project_name}"));
        let author = options.author;
        let host = options.host.unwrap_or_else(|| "localhost".to_string());
        let port = options.port.unwrap_or(8080);
        let enable_git = options.enable_git.unwrap_or(true);
        let enable_precommit = options.enable_precommit.unwrap_or(true);
        let enable_swagger = options.enable_swagger.unwrap_or(true);
        let enable_cors = options.enable_cors.unwrap_or(true);
        let enable_jwt = options.enable_jwt.unwrap_or(false);
        let enable_redis = options.enable_redis.unwrap_or(false);
        let database_type = options.database_type;

        // 1. 框架级别生成 (Gin) - 首先生成应用结构
        let mut project_params = ProjectParams::new(project_name.clone())
            .with_description(description.clone())
            .with_license(license.clone());
        if let Some(author) = author.clone() {
            project_params = project_params.with_author(author);
        }

        let go_params = GoParams::new(module_name.clone()).with_version(go_version.clone());

        let mut gin_params = GinParams::from_project_name(project_name.clone())
            .with_project(project_params)
            .with_go(go_params)
            .with_server(host, port)
            .with_swagger(enable_swagger)
            .with_cors(enable_cors)
            .with_jwt(enable_jwt)
            .with_precommit(enable_precommit);

        if let Some(db_type) = database_type {
            gin_params = gin_params.with_database(db_type);
        }

        if enable_redis {
            gin_params = gin_params.with_redis(true);
        }

        self.gin_generator
            .generate(gin_params.clone(), output_path)
            .context("Failed to generate Gin framework files")?;

        // 2. 语言级别生成 (Go) - 然后执行 go mod init 和 go mod tidy
        let go_params = GoParams::new(module_name).with_version(go_version);

        self.go_generator
            .generate(go_params, output_path)
            .context("Failed to generate Go files")?;

        // 3. 项目级别生成 - 最后执行 git init 等项目级操作
        let mut project_params = ProjectParams::new(project_name.clone())
            .with_license(license)
            .with_git(enable_git)
            .with_precommit(enable_precommit)
            .with_description(description);

        if let Some(author) = author {
            project_params = project_params.with_author(author);
        }

        self.project_generator
            .generate(project_params, output_path)
            .context("Failed to generate project files")?;

        // 4. 执行后处理逻辑 - 在所有生成完成后执行 post_process
        self.gin_generator
            .post_process(&gin_params, output_path)
            .context("Failed to execute Gin post-processing")?;

        tracing::info!("Gin project generation completed successfully!");
        tracing::info!("Project created at: {}", output_path.display());

        Ok(())
    }

    /// 语言级别生成 (Python) - 使用 uv init 创建项目。项目级尾段由 run_project_step 处理。
    async fn generate_python_language(
        &mut self,
        project_name: String,
        output_path: &Path,
        enable_precommit: bool,
    ) -> Result<()> {
        tracing::info!("Starting Python project generation: {project_name}");

        // 获取实际的 uv 版本和 Python 版本
        let env_checker = EnvironmentChecker::new();

        let uv_version = env_checker
            .get_uv_version()
            .await
            .unwrap_or_else(|_| "uv 0.9.5".to_string());

        // 从 "uv x.y.z" 格式中提取版本号
        let uv_version = uv_version
            .strip_prefix("uv ")
            .unwrap_or(&uv_version)
            .trim()
            .to_string();

        // 获取系统 Python 版本，如果获取失败则使用默认值
        let python_version = env_checker
            .get_python_version()
            .await
            .unwrap_or_else(|_| "3.12".to_string());

        let python_params = PythonParams::new(project_name.clone())
            .with_version(python_version)
            .with_uv_version(uv_version)
            .with_precommit(enable_precommit);

        self.python_generator
            .generate(python_params, output_path)
            .context("Failed to generate Python files")?;

        Ok(())
    }

    /// 框架级别生成 (FastAPI) - 纯内嵌模板渲染（不调用 uv）。
    ///
    /// 与基础 Python 路径不同：FastAPI 是配置驱动的完整项目，pyproject/main/config
    /// 全部由模板生成，无需 `uv init`/`uv add`/`uv sync`（这也让渲染可离线、可测试）。
    /// host/port 写入模板上下文，生成的 config.toml 即据此驱动监听地址。
    async fn generate_fastapi_language(
        &mut self,
        project_name: String,
        output_path: &Path,
        host: Option<String>,
        port: Option<u16>,
        enable_precommit: bool,
    ) -> Result<()> {
        tracing::info!("Starting FastAPI project generation: {project_name}");

        let env_checker = EnvironmentChecker::new();
        let python_version = env_checker
            .get_python_version()
            .await
            .unwrap_or_else(|_| "3.12".to_string());

        let mut python_params = PythonParams::new(project_name.clone())
            .with_version(python_version)
            .with_precommit(enable_precommit);
        python_params.base.host = Some(host.unwrap_or_else(|| "0.0.0.0".to_string()));
        python_params.base.port = Some(port.unwrap_or(8080));

        let template_path = "frameworks/python/fastapi";
        if !crate::template_engine::embedded_template_dir_exists(template_path) {
            return Err(anyhow::anyhow!(
                "FastAPI embedded templates not found at: {template_path}"
            ));
        }

        let context = python_params.to_template_context();
        let mut template_processor = TemplateProcessor::new()?;
        template_processor
            .process_embedded_template_directory(template_path, output_path, context)
            .context("Failed to generate FastAPI files")?;

        tracing::info!("FastAPI structure generated");
        Ok(())
    }

    /// 框架级别生成 (Python MCP Server) - 纯内嵌模板渲染（可离线、可测试）。
    ///
    /// 与 FastAPI 同形：配置驱动的完整项目，全部由模板渲染，无需 `uv init`/`uv sync`。
    /// host/port 写入上下文（端口规范为 8000，区别于 FastAPI 的 8080）；`mcp_backend` /
    /// `mcp_backend_is_official` 在 `to_template_context()` 之后注入，驱动模板按后端分支渲染；
    /// `auth_mode` / `auth_enabled` 同样在其后注入，驱动可选 JWT 鉴权代码的渲染。
    #[allow(clippy::too_many_arguments)]
    async fn generate_mcp_python_language(
        &mut self,
        project_name: String,
        output_path: &Path,
        host: Option<String>,
        port: Option<u16>,
        enable_precommit: bool,
        mcp_options: McpPythonOptions,
    ) -> Result<()> {
        tracing::info!("Starting Python MCP server generation: {project_name}");

        let McpPythonOptions { backend, auth_mode } = mcp_options;

        let env_checker = EnvironmentChecker::new();
        let python_version = env_checker
            .get_python_version()
            .await
            .unwrap_or_else(|_| "3.12".to_string());

        let mut python_params = PythonParams::new(project_name.clone())
            .with_version(python_version)
            .with_precommit(enable_precommit);
        python_params.base.host = Some(host.unwrap_or_else(|| "0.0.0.0".to_string()));
        python_params.base.port = Some(port.unwrap_or(8000));

        let template_path = "frameworks/python/mcp-python";
        if !crate::template_engine::embedded_template_dir_exists(template_path) {
            return Err(anyhow::anyhow!(
                "mcp-python embedded templates not found at: {template_path}"
            ));
        }

        let mut context = python_params.to_template_context();
        context.insert(
            "mcp_backend".to_string(),
            serde_json::json!(backend.as_str()),
        );
        context.insert(
            "mcp_backend_is_official".to_string(),
            serde_json::json!(backend.is_official()),
        );
        context.insert(
            "auth_mode".to_string(),
            serde_json::json!(auth_mode.as_str()),
        );
        context.insert(
            "auth_enabled".to_string(),
            serde_json::json!(auth_mode.is_enabled()),
        );
        context.insert(
            "auth_is_azure_ad".to_string(),
            serde_json::json!(auth_mode.is_azure_ad()),
        );

        let mut template_processor = TemplateProcessor::new()?;
        template_processor
            .process_embedded_template_directory(template_path, output_path, context)
            .context("Failed to generate mcp-python files")?;

        tracing::info!("mcp-python structure generated");
        Ok(())
    }

    /// 语言级别生成 (Rust) - 使用模板创建项目。项目级尾段由 run_project_step 处理。
    async fn generate_rust_language(
        &mut self,
        project_name: String,
        output_path: &Path,
        enable_precommit: bool,
        enable_proto_gen: bool,
        enable_error_gen: bool,
    ) -> Result<()> {
        tracing::info!("Starting Rust project generation: {project_name}");

        // 获取实际的 Rust 版本
        let env_checker = EnvironmentChecker::new();
        let rust_version = env_checker
            .get_rust_version()
            .await
            .unwrap_or_else(|_| crate::constants::defaults::RUST_VERSION.to_string());

        let mut rust_params = RustParams::new(project_name.clone())
            .with_rust_version(rust_version)
            .with_proto_gen(enable_proto_gen)
            .with_error_gen(enable_error_gen);
        rust_params.base.enable_precommit = enable_precommit;

        RustGenerator::new()?
            .generate(rust_params, output_path)
            .context("Failed to generate Rust files")?;

        Ok(())
    }

    async fn generate_vue3_embedded(
        &mut self,
        project_name: String,
        output_path: &Path,
    ) -> Result<()> {
        tracing::info!("Starting Vue3 project generation: {project_name}");

        let vue3_params =
            crate::generators::framework::vue3::Vue3Params::from_project_name(project_name.clone());

        let template_path = "frameworks/typescript/vue3";
        if !crate::template_engine::embedded_template_dir_exists(template_path) {
            return Err(anyhow::anyhow!(
                "Vue3 embedded templates not found at: {template_path}"
            ));
        }

        let context = vue3_params.to_template_context();
        let mut template_processor = TemplateProcessor::new()?;

        template_processor
            .process_embedded_template_directory(template_path, output_path, context)
            .context("Failed to generate Vue3 files")?;

        tracing::info!("Vue3 structure generated");

        if crate::utils::toolchain::tool_available("pnpm") {
            tracing::info!("📦 Installing frontend dependencies...");
            let outcome = crate::utils::toolchain::ExternalCommand::new("pnpm")
                .arg("install")
                .current_dir(output_path)
                .run()
                .context("Failed to execute pnpm install")?;

            if outcome.success() {
                tracing::info!("✅ Dependencies installed successfully");
            } else {
                tracing::warn!("⚠️ Warning: pnpm install failed: {}", outcome.stderr());
            }
        } else {
            tracing::warn!(
                "⚠️ Warning: pnpm is not installed. Please run `pnpm install` manually in the project directory."
            );
        }

        Ok(())
    }

    /// 框架级别生成 (Go MCP Server) - 纯内嵌模板渲染（可离线、可测试）。
    ///
    /// 复用 GoParams 上下文得到 module_name / go_version / project_name_snake；
    /// host/port 写入上下文，生成的 config.toml 即据此驱动监听地址。
    /// 渲染后若本机有 go 则尝试 `go mod tidy`（非致命，失败仅告警）。
    async fn generate_mcp_server_embedded(
        &mut self,
        project_name: String,
        output_path: &Path,
        host: Option<String>,
        port: Option<u16>,
    ) -> Result<()> {
        tracing::info!("Starting Go MCP server generation: {project_name}");

        let mut go_params =
            GoParams::from_project_name(project_name).with_version("1.24".to_string());
        go_params.base.host = Some(host.unwrap_or_else(|| "0.0.0.0".to_string()));
        go_params.base.port = Some(port.unwrap_or(8080));

        let template_path = "frameworks/go/mcp-server";
        if !crate::template_engine::embedded_template_dir_exists(template_path) {
            return Err(anyhow::anyhow!(
                "MCP server embedded templates not found at: {template_path}"
            ));
        }

        let context = go_params.to_template_context();
        let mut template_processor = TemplateProcessor::new()?;
        template_processor
            .process_embedded_template_directory(template_path, output_path, context)
            .context("Failed to generate MCP server files")?;

        tracing::info!("MCP server structure generated");

        if crate::utils::toolchain::tool_available("go") {
            tracing::info!("📦 Running go mod tidy...");
            let outcome = crate::utils::toolchain::ExternalCommand::new("go")
                .arg("mod")
                .arg("tidy")
                .current_dir(output_path)
                .run()
                .context("Failed to execute go mod tidy")?;

            if outcome.success() {
                tracing::info!("✅ go mod tidy completed");
            } else {
                tracing::warn!(
                    "⚠️ Warning: go mod tidy failed (run `make generate && go mod tidy` after editing proto): {}",
                    outcome.stderr()
                );
            }
        } else {
            tracing::warn!(
                "⚠️ Warning: go is not installed. Run `make generate` then `go mod tidy` manually."
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::registry;
    use std::fs;
    use walkdir::WalkDir;

    fn request_for<'a>(
        language: Language,
        framework: Framework,
        project_name: &str,
        output_path: &'a Path,
        enable_build: bool,
    ) -> GenerationRequest<'a> {
        let spec = registry::resolve(language, framework).expect("spec resolves");
        GenerationRequest {
            spec,
            project_name: project_name.to_string(),
            output_path,
            license: "MIT".to_string(),
            enable_precommit: false,
            enable_proto_gen: false,
            enable_error_gen: false,
            enable_build,
            gin_options: GinProjectOptions::new(),
            mcp_backend: McpBackend::Fastmcp,
            auth_mode: AuthMode::None,
        }
    }

    fn relative_files(root: &Path) -> Vec<String> {
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
    fn render_build_tooling_emits_makefile_and_dockerfile_for_python() {
        // Given: a Python(None) request with --with-build enabled
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut orchestrator = GeneratorOrchestrator::new().expect("orchestrator");
        let request = request_for(
            Language::Python,
            Framework::None,
            "build-probe",
            tmp.path(),
            true,
        );

        // When: only the build step runs (no language/project shell-out)
        orchestrator
            .render_build_tooling(&request)
            .expect("render build tooling");

        // Then: Makefile + Dockerfile exist, project_name substituted, no residual delimiters
        let files = relative_files(tmp.path());
        assert!(
            files.iter().any(|f| f == "Makefile"),
            "expected Makefile, got: {files:?}"
        );
        assert!(
            files.iter().any(|f| f == "Dockerfile"),
            "expected Dockerfile, got: {files:?}"
        );
        let makefile = fs::read_to_string(tmp.path().join("Makefile")).expect("read Makefile");
        assert!(
            makefile.contains("build-probe"),
            "project_name not substituted into Makefile:\n{makefile}"
        );
        for rel in &files {
            let content =
                fs::read_to_string(tmp.path().join(rel)).unwrap_or_else(|_| panic!("read {rel}"));
            assert!(!content.contains("<<"), "file {rel} has unrendered `<<`");
            assert!(!content.contains("%>"), "file {rel} has unrendered `%>`");
        }
    }

    #[test]
    fn render_build_tooling_emits_makefile_and_dockerfile_for_rust() {
        // Given: a Rust(None) request with --with-build enabled
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut orchestrator = GeneratorOrchestrator::new().expect("orchestrator");
        let request = request_for(Language::Rust, Framework::None, "rusty", tmp.path(), true);

        // When
        orchestrator
            .render_build_tooling(&request)
            .expect("render build tooling");

        // Then
        let files = relative_files(tmp.path());
        assert!(files.iter().any(|f| f == "Makefile"), "got: {files:?}");
        assert!(files.iter().any(|f| f == "Dockerfile"), "got: {files:?}");
    }
}
