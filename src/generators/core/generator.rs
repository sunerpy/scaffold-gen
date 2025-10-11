use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

use super::parameters::Parameters;
use super::template_processor::TemplateProcessor;

/// 核心生成器trait，定义所有生成器的基础接口
pub trait Generator {
    /// 生成器参数类型
    type Params: Parameters;

    /// 获取生成器名称
    fn name(&self) -> &'static str;

    /// 获取生成器描述 (预留给未来的CLI帮助信息显示)
    #[allow(dead_code)]
    fn description(&self) -> Option<&'static str> {
        None
    }

    /// 获取模板路径（相对于templates目录）
    fn get_template_path(&self) -> &'static str;

    /// 生成代码 - 默认实现使用模板渲染
    fn generate(&mut self, params: Self::Params, output_path: &Path) -> Result<()> {
        let template_processor = TemplateProcessor::new()?;
        let template_path = self.get_template_path();
        let context = params.to_template_context();

        println!("🔧 Generating {} structure", self.name());

        if template_processor.template_exists(template_path) {
            self.render_templates(
                &template_processor,
                template_path,
                output_path,
                context,
                &params,
            )?;
        } else {
            return Err(anyhow::anyhow!(
                "{} templates not found at: {}",
                self.name(),
                template_path
            ));
        }

        println!("✅ {} structure generated", self.name());
        Ok(())
    }

    /// 渲染模板 - 可以被子类重写以实现自定义逻辑
    fn render_templates(
        &mut self,
        template_processor: &TemplateProcessor,
        template_path: &str,
        output_path: &Path,
        context: HashMap<String, Value>,
        _params: &Self::Params,
    ) -> Result<()> {
        // 默认实现：直接处理模板目录
        let template_path_obj = std::path::Path::new(template_path);
        template_processor.process_template_directory(template_path_obj, output_path, context)
    }

    /// 后处理逻辑，在生成完成后执行
    fn post_process(&mut self, _params: &Self::Params, _output_path: &Path) -> Result<()> {
        // 默认实现为空
        Ok(())
    }
}

/// 项目级别生成器trait
pub trait ProjectGenerator: Generator {
    /// 生成许可证文件
    fn generate_license(&mut self, params: &Self::Params, output_path: &Path) -> Result<()>;

    /// 初始化Git仓库
    fn init_git_repository(&mut self, output_path: &Path) -> Result<()>;

    /// 生成README文件 (预留给未来的文档生成功能)
    #[allow(dead_code)]
    fn generate_readme(&mut self, params: &Self::Params, output_path: &Path) -> Result<()>;

    /// 安装 pre-commit hooks
    fn install_precommit(&mut self, output_path: &Path) -> Result<()>;
}

/// 语言级别生成器trait (预留给未来的多语言支持扩展)
pub trait LanguageGenerator: Generator {
    /// 获取语言名称 (预留给未来的语言识别功能)
    #[allow(dead_code)]
    fn language(&self) -> &'static str;

    /// 设置语言环境 (预留给未来的环境配置功能)
    #[allow(dead_code)]
    fn setup_environment(&mut self, params: &Self::Params, output_path: &Path) -> Result<()>;

    /// 生成语言配置文件 (预留给未来的语言特定配置生成)
    #[allow(dead_code)]
    fn generate_language_config(&mut self, params: &Self::Params, output_path: &Path)
    -> Result<()>;
}

/// 框架级别生成器trait
pub trait FrameworkGenerator: Generator {
    /// 获取框架名称
    #[allow(dead_code)]
    fn framework(&self) -> &'static str;

    /// 获取支持的语言
    #[allow(dead_code)]
    fn language(&self) -> &'static str;

    /// 生成基础结构
    fn generate_basic_structure(&mut self, params: &Self::Params, output_path: &Path)
    -> Result<()>;

    /// 生成配置文件
    #[allow(dead_code)]
    fn generate_config(&mut self, params: &Self::Params, output_path: &Path) -> Result<()>;

    /// 生成中间件
    #[allow(dead_code)]
    fn generate_middleware(&mut self, params: &Self::Params, output_path: &Path) -> Result<()>;
}
