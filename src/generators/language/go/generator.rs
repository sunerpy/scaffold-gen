use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use super::parameters::GoParams;
use crate::constants::{Framework, Language};
use crate::generators::core::{
    Generator, InheritableParams, LanguageGenerator as LanguageGeneratorTrait, Parameters,
    TemplateProcessor,
};
use crate::utils::go_tools::GoTools;

/// Go语言级别生成器实现
pub struct GoGenerator {
    template_processor: TemplateProcessor,
}

impl GoGenerator {
    /// 创建新的Go生成器
    pub fn new() -> Result<Self> {
        Ok(Self {
            template_processor: TemplateProcessor::new()?,
        })
    }

    /// 检查Go是否已安装
    fn check_go_installation(&self) -> Result<String> {
        let output = Command::new("go")
            .args(["version"])
            .output()
            .context("Failed to check Go installation")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Go is not installed or not in PATH"));
        }

        let version_output = String::from_utf8_lossy(&output.stdout);
        Ok(version_output.trim().to_string())
    }

    /// 初始化Go模块
    fn init_go_module(&self, params: &GoParams, output_path: &Path) -> Result<()> {
        // 使用项目名而不是完整的模块名
        let project_name = &params.base_params().project_name;

        // 尝试运行 go mod init
        let output = Command::new("go")
            .args(["mod", "init", project_name])
            .current_dir(output_path)
            .output();

        match output {
            Ok(result) if result.status.success() => {
                println!("Go module initialized: {project_name}");
                Ok(())
            }
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                eprintln!("go mod init failed: {stderr}");

                // 手动创建 go.mod 文件
                let go_mod_content = format!("module {project_name}\n\ngo 1.21\n");
                let go_mod_path = output_path.join("go.mod");
                std::fs::write(&go_mod_path, go_mod_content)?;
                println!("Manually created go.mod file");
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to execute go mod init: {e}");

                // 手动创建 go.mod 文件
                let go_mod_content = format!("module {project_name}\n\ngo 1.21\n");
                let go_mod_path = output_path.join("go.mod");
                std::fs::write(&go_mod_path, go_mod_content)?;
                println!("Manually created go.mod file");
                Ok(())
            }
        }
    }

    /// 设置依赖
    fn setup_dependencies(&self, output_path: &Path) -> Result<()> {
        match GoTools::mod_tidy(output_path) {
            Ok(_) => {
                println!("Dependencies organized with go mod tidy");
                Ok(())
            }
            Err(e) => {
                eprintln!("Warning: go mod tidy failed: {e}");
                // 不返回错误，因为这不是致命的
                Ok(())
            }
        }
    }
}

impl Default for GoGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create GoGenerator")
    }
}

impl Generator for GoGenerator {
    type Params = GoParams;

    fn name(&self) -> &'static str {
        "Go Language"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Go language project generator")
    }

    fn get_template_path(&self) -> &'static str {
        "languages/go"
    }

    fn generate(&mut self, params: Self::Params, output_path: &Path) -> Result<()> {
        // 验证参数
        params.validate()?;

        // 检查Go安装
        self.check_go_installation()?;

        // 处理嵌入式模板
        let mut template_processor = TemplateProcessor::new()?;
        let template_path = self.get_template_path();
        let context = params.to_template_context();

        println!("Generating {} structure", self.name());

        // 检查嵌入式模板目录是否存在
        if crate::template_engine::embedded_template_dir_exists(template_path) {
            template_processor.process_embedded_template_directory(
                template_path,
                output_path,
                context,
            )?;
        } else {
            return Err(anyhow::anyhow!(
                "{} embedded templates not found at: {}",
                self.name(),
                template_path
            ));
        }

        // 初始化Go模块
        self.init_go_module(&params, output_path)?;

        // 设置依赖
        self.setup_dependencies(output_path)?;

        println!("Go language generation completed successfully");
        Ok(())
    }
}

impl LanguageGeneratorTrait for GoGenerator {
    fn language(&self) -> &'static str {
        Language::Go.as_str()
    }

    fn setup_environment(&mut self, params: &Self::Params, output_path: &Path) -> Result<()> {
        // 初始化Go模块
        if params.enable_modules() {
            self.init_go_module(params, output_path)?;
        }

        // 整理依赖
        self.setup_dependencies(output_path)?;

        Ok(())
    }

    fn generate_language_config(
        &mut self,
        params: &Self::Params,
        output_path: &Path,
    ) -> Result<()> {
        // 如果启用了Go modules，确保go.mod文件存在
        if params.enable_modules() {
            let go_mod_path = output_path.join("go.mod");
            if !go_mod_path.exists() {
                self.init_go_module(params, output_path)?;
            }
        }

        Ok(())
    }
}
