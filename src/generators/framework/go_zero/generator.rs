use anyhow::{Context, Result};
use std::path::Path;

use super::parameters::GoZeroParams;
use crate::constants::{Framework, Language};
use crate::generators::core::{
    FrameworkGenerator as FrameworkGeneratorTrait, Generator, Parameters, TemplateProcessor,
};

pub struct GoZeroGenerator {
    template_processor: TemplateProcessor,
}

impl GoZeroGenerator {
    pub fn new() -> Result<Self> {
        Ok(Self {
            template_processor: TemplateProcessor::new()?,
        })
    }
}

impl Default for GoZeroGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create GoZeroGenerator")
    }
}

impl Generator for GoZeroGenerator {
    type Params = GoZeroParams;

    fn name(&self) -> &'static str {
        "go-zero"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Generates Go-Zero microservice framework specific files and structure")
    }

    fn get_template_path(&self) -> &'static str {
        "frameworks/go/go_zero"
    }

    fn generate(&mut self, params: Self::Params, output_path: &Path) -> Result<()> {
        params.validate()?;

        println!("Generating Go-Zero microservice framework structure");

        // 处理Go-Zero模板目录
        let template_dir = "frameworks/go/go_zero";
        if self.template_processor.template_exists(template_dir) {
            let context = params.to_template_context();

            let template_path = std::path::Path::new(template_dir);
            self.template_processor
                .process_template_directory(template_path, output_path, context)
                .context("Failed to process Go-Zero templates")?;
        } else {
            // 如果模板不存在，生成基础结构
            self.generate_basic_structure(&params, output_path)?;
        }

        println!("Go-Zero microservice framework structure generated");
        Ok(())
    }
}

impl FrameworkGeneratorTrait for GoZeroGenerator {
    fn framework(&self) -> &'static str {
        Framework::GoZero.as_str()
    }

    fn language(&self) -> &'static str {
        Language::Go.as_str()
    }

    fn generate_basic_structure(
        &mut self,
        params: &Self::Params,
        output_path: &Path,
    ) -> Result<()> {
        // 创建基础目录结构
        let dirs = ["api", "rpc", "admin", "common", "model"];

        for dir in &dirs {
            let dir_path = output_path.join(dir);
            std::fs::create_dir_all(&dir_path)
                .with_context(|| format!("Failed to create directory: {}", dir_path.display()))?;
        }

        // 根据参数决定生成哪些服务
        if params.enable_api {
            self.generate_api_service(params, output_path)?;
        }

        if params.enable_rpc {
            self.generate_rpc_service(params, output_path)?;
        }

        if params.enable_admin {
            self.generate_admin_service(params, output_path)?;
        }

        Ok(())
    }

    fn generate_config(&mut self, _params: &Self::Params, _output_path: &Path) -> Result<()> {
        // Go-Zero 配置生成逻辑
        Ok(())
    }

    fn generate_middleware(&mut self, _params: &Self::Params, _output_path: &Path) -> Result<()> {
        // Go-Zero 中间件生成逻辑
        Ok(())
    }
}

impl GoZeroGenerator {
    fn generate_api_service(&self, _params: &GoZeroParams, _output_path: &Path) -> Result<()> {
        // 生成API服务相关文件
        Ok(())
    }

    fn generate_rpc_service(&self, _params: &GoZeroParams, _output_path: &Path) -> Result<()> {
        // 生成RPC服务相关文件
        Ok(())
    }

    fn generate_admin_service(&self, _params: &GoZeroParams, _output_path: &Path) -> Result<()> {
        // 生成管理后台相关文件
        Ok(())
    }
}
