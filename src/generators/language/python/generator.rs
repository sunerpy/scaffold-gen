use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::constants::defaults;
use crate::generators::core::{Generator, InheritableParams, Parameters, TemplateProcessor};
use crate::generators::language::python::parameters::PythonParams;

fn rewrite_requires_python(output_path: &Path) -> Result<()> {
    let pyproject_path = output_path.join("pyproject.toml");
    let content = std::fs::read_to_string(&pyproject_path)
        .with_context(|| format!("Failed to read {}", pyproject_path.display()))?;
    let requirement = regex::Regex::new(r#"(?m)^requires-python[ \t]*=[ \t]*"[^"]*"[ \t]*$"#)
        .context("Failed to compile requires-python pattern")?;

    if !requirement.is_match(&content) {
        return Err(anyhow::anyhow!(
            "Missing requires-python in {}",
            pyproject_path.display()
        ));
    }

    let replacement = format!("requires-python = \">={}\"", defaults::PYTHON_MIN_VERSION);
    let rewritten = requirement.replace(&content, replacement).into_owned();
    std::fs::write(&pyproject_path, rewritten)
        .with_context(|| format!("Failed to write {}", pyproject_path.display()))
}

/// Python 语言生成器
pub struct PythonGenerator {}

impl PythonGenerator {
    /// 创建新的 Python 生成器
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// 使用 uv init 初始化项目
    fn init_uv_project(&self, params: &PythonParams, output_path: &Path) -> Result<()> {
        tracing::debug!("Initializing Python project with uv...");

        let project_name = &params.base_params().project_name;

        // 使用 uv init 创建项目
        let status = Command::new("uv")
            .arg("init")
            .arg("--name")
            .arg(project_name)
            .arg(output_path)
            .env_remove("VIRTUAL_ENV")
            .status()
            .context("Failed to execute uv init")?;

        if !status.success() {
            return Err(anyhow::anyhow!("uv init failed"));
        }

        tracing::debug!("Python project initialized with uv");
        Ok(())
    }

    /// 添加必要的依赖
    fn add_dependencies(&self, output_path: &Path) -> Result<()> {
        tracing::debug!("Adding Python dependencies...");

        let dependencies = vec!["pydantic", "python-dotenv", "structlog", "rich"];

        for dep in dependencies {
            let status = Command::new("uv")
                .arg("add")
                .arg(dep)
                .env_remove("VIRTUAL_ENV")
                .current_dir(output_path)
                .status()
                .context(format!("Failed to add dependency: {}", dep))?;

            if !status.success() {
                tracing::warn!("Warning: Failed to add dependency {}", dep);
            }
        }

        tracing::debug!("Dependencies added successfully");
        Ok(())
    }

    /// 安装依赖
    fn install_dependencies(&self, output_path: &Path) -> Result<()> {
        tracing::debug!("Installing Python dependencies...");

        let status = Command::new("uv")
            .arg("sync")
            .env_remove("VIRTUAL_ENV")
            .current_dir(output_path)
            .status()
            .context("Failed to execute uv sync")?;

        if !status.success() {
            tracing::warn!("Warning: uv sync failed, you may need to run it manually");
        } else {
            tracing::debug!("Python dependencies installed successfully");
        }

        Ok(())
    }

    /// Generate the pure-Python project, merging `context_overrides` on top of the
    /// params-derived template context. The orchestrator uses this to inject
    /// `enable_build` / `docker_image_name` without adding fields to PythonParams.
    /// `Generator::generate` calls this with an empty override map.
    pub fn generate_with_context(
        &mut self,
        params: PythonParams,
        output_path: &Path,
        context_overrides: HashMap<String, Value>,
    ) -> Result<()> {
        params.validate()?;

        tracing::info!("Generating {} structure", self.name());

        self.init_uv_project(&params, output_path)?;
        rewrite_requires_python(output_path)?;

        let mut template_processor = TemplateProcessor::new()?;
        let template_path = self.get_template_path();
        let mut context = params.to_template_context();
        context.extend(context_overrides);

        if crate::template_engine::embedded_template_dir_exists(template_path) {
            template_processor.process_embedded_template_directory(
                template_path,
                output_path,
                context,
            )?;
        } else {
            tracing::warn!(
                "Warning: {} embedded templates not found at: {}",
                self.name(),
                template_path
            );
        }

        self.add_dependencies(output_path)?;
        self.install_dependencies(output_path)?;

        tracing::info!("Python language generation completed successfully");
        Ok(())
    }
}

impl Generator for PythonGenerator {
    type Params = PythonParams;

    fn name(&self) -> &'static str {
        "Python Language"
    }

    fn get_template_path(&self) -> &'static str {
        "languages/python"
    }

    fn generate(&mut self, params: Self::Params, output_path: &Path) -> Result<()> {
        self.generate_with_context(params, output_path, HashMap::new())
    }
}

#[cfg(test)]
#[path = "generator_tests.rs"]
mod tests;
