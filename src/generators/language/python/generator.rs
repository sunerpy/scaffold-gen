use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::constants::Language;
use crate::generators::core::{
    Generator, InheritableParams, LanguageGenerator as LanguageGeneratorTrait, Parameters,
    TemplateProcessor,
};
use crate::generators::language::python::parameters::PythonParams;

/// Python 语言生成器
pub struct PythonGenerator {}

impl PythonGenerator {
    /// 创建新的 Python 生成器
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// 使用 uv init 初始化项目
    fn init_uv_project(&self, params: &PythonParams, output_path: &Path) -> Result<()> {
        println!("Initializing Python project with uv...");

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

        println!("Python project initialized with uv");
        Ok(())
    }

    /// 添加必要的依赖
    fn add_dependencies(&self, output_path: &Path) -> Result<()> {
        println!("Adding Python dependencies...");

        let dependencies = vec!["pydantic", "python-dotenv", "rich"];

        for dep in dependencies {
            let status = Command::new("uv")
                .arg("add")
                .arg(dep)
                .env_remove("VIRTUAL_ENV")
                .current_dir(output_path)
                .status()
                .context(format!("Failed to add dependency: {}", dep))?;

            if !status.success() {
                println!("Warning: Failed to add dependency {}", dep);
            }
        }

        println!("Dependencies added successfully");
        Ok(())
    }

    /// 安装依赖
    fn install_dependencies(&self, output_path: &Path) -> Result<()> {
        println!("Installing Python dependencies...");

        let status = Command::new("uv")
            .arg("sync")
            .env_remove("VIRTUAL_ENV")
            .current_dir(output_path)
            .status()
            .context("Failed to execute uv sync")?;

        if !status.success() {
            println!("Warning: uv sync failed, you may need to run it manually");
        } else {
            println!("Python dependencies installed successfully");
        }

        Ok(())
    }
}

impl Default for PythonGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create PythonGenerator")
    }
}

impl Generator for PythonGenerator {
    type Params = PythonParams;

    fn name(&self) -> &'static str {
        "Python Language"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Python language project generator")
    }

    fn get_template_path(&self) -> &'static str {
        "languages/python"
    }

    fn generate(&mut self, params: Self::Params, output_path: &Path) -> Result<()> {
        // 验证参数
        params.validate()?;

        println!("Generating {} structure", self.name());

        // 1. 使用 uv init 创建基础项目结构
        self.init_uv_project(&params, output_path)?;

        // 2. 处理嵌入式模板
        let mut template_processor = TemplateProcessor::new()?;
        let template_path = self.get_template_path();
        let context = params.to_template_context();

        // 检查嵌入式模板目录是否存在
        if crate::template_engine::embedded_template_dir_exists(template_path) {
            template_processor.process_embedded_template_directory(
                template_path,
                output_path,
                context,
            )?;
        } else {
            println!(
                "Warning: {} embedded templates not found at: {}",
                self.name(),
                template_path
            );
        }

        // 3. 添加必要的依赖
        self.add_dependencies(output_path)?;

        // 4. 安装依赖
        self.install_dependencies(output_path)?;

        println!("Python language generation completed successfully");
        Ok(())
    }
}

impl LanguageGeneratorTrait for PythonGenerator {
    fn language(&self) -> &'static str {
        Language::Python.as_str()
    }

    fn setup_environment(&mut self, params: &Self::Params, output_path: &Path) -> Result<()> {
        // 初始化 Python 项目
        self.init_uv_project(params, output_path)?;

        // 安装依赖
        self.install_dependencies(output_path)?;

        Ok(())
    }

    fn generate_language_config(
        &mut self,
        params: &Self::Params,
        output_path: &Path,
    ) -> Result<()> {
        // 确保 pyproject.toml 文件存在
        let pyproject_path = output_path.join("pyproject.toml");
        if !pyproject_path.exists() {
            self.init_uv_project(params, output_path)?;
        }

        Ok(())
    }
}
