use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use super::parameters::GoParams;
use crate::constants::Language;
use crate::generators::core::{
    Generator, LanguageGenerator as LanguageGeneratorTrait, Parameters, TemplateProcessor,
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
    fn init_go_module(&mut self, params: &GoParams, output_path: &Path) -> Result<()> {
        // 从模块名称中提取项目名称（取最后一部分）
        let project_name = params
            .module_name
            .split('/')
            .next_back()
            .unwrap_or(&params.module_name);

        // 使用go mod init命令初始化模块
        let status = Command::new("go")
            .args(["mod", "init", project_name])
            .current_dir(output_path)
            .status();

        match status {
            Ok(status) if status.success() => {
                println!("✅ Initialized Go module: {project_name}");
            }
            _ => {
                // 如果go命令失败，手动创建go.mod文件
                let go_mod_content = format!("module {}\n\ngo {}\n", project_name, params.version);

                let go_mod_path = output_path.join("go.mod");
                std::fs::write(&go_mod_path, go_mod_content)
                    .context("Failed to create go.mod file")?;

                println!("✅ Created go.mod file manually");
            }
        }

        Ok(())
    }

    /// 设置依赖
    fn setup_dependencies(&mut self, _params: &GoParams, output_path: &Path) -> Result<()> {
        GoTools::mod_tidy(output_path)
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
        match self.check_go_installation() {
            Ok(version) => println!("✅ Go {version} detected"),
            Err(_) => {
                println!("⚠️  Warning: Go not found in PATH, generated files may need manual setup")
            }
        }

        // 创建新的模板处理器实例避免借用冲突
        let template_path = self.get_template_path();
        let context = params.to_template_context();

        println!("🔧 Generating {} structure", self.name());

        if self.template_processor.template_exists(template_path) {
            // 创建新的模板处理器实例避免借用冲突
            let template_processor = TemplateProcessor::new()?;
            self.render_templates(
                &template_processor,
                template_path,
                output_path,
                context,
                &params,
            )?;
        } else {
            println!(
                "⚠️  {} templates not found at: {}, generating basic structure",
                self.name(),
                template_path
            );
        }

        // 执行语言级别的后处理步骤
        println!("🔧 Setting up Go module and dependencies...");

        // 1. 初始化 Go 模块
        self.init_go_module(&params, output_path)?;

        // 2. 整理依赖
        self.setup_dependencies(&params, output_path)?;

        println!("✅ {} structure generated", self.name());
        Ok(())
    }
}

impl LanguageGeneratorTrait for GoGenerator {
    fn language(&self) -> &'static str {
        Language::Go.as_str()
    }

    fn setup_environment(&mut self, params: &Self::Params, output_path: &Path) -> Result<()> {
        // 初始化Go模块
        if params.enable_modules {
            self.init_go_module(params, output_path)?;
        }

        // 整理依赖
        self.setup_dependencies(params, output_path)?;

        Ok(())
    }

    fn generate_language_config(
        &mut self,
        params: &Self::Params,
        output_path: &Path,
    ) -> Result<()> {
        // 生成 go.mod 文件
        if params.enable_modules {
            self.init_go_module(params, output_path)?;
        }

        Ok(())
    }
}
