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

    /// 获取模板路径（相对于templates目录）
    fn get_template_path(&self) -> &'static str;

    /// 生成代码 - 默认实现使用嵌入式模板渲染
    fn generate(&mut self, params: Self::Params, output_path: &Path) -> Result<()> {
        let mut template_processor = TemplateProcessor::new()?;
        let template_path = self.get_template_path();
        let context = params.to_template_context();

        tracing::info!("Generating {} structure", self.name());

        // 检查嵌入式模板目录是否存在
        if crate::template_engine::embedded_template_dir_exists(template_path) {
            self.render_embedded_templates(
                &mut template_processor,
                template_path,
                output_path,
                context,
                &params,
            )?;
        } else {
            return Err(anyhow::anyhow!(
                "{} embedded templates not found at: {}",
                self.name(),
                template_path
            ));
        }

        tracing::info!("{} structure generated", self.name());
        Ok(())
    }

    /// 渲染嵌入式模板 - 可以被子类重写以实现自定义逻辑
    fn render_embedded_templates(
        &mut self,
        template_processor: &mut TemplateProcessor,
        template_path: &str,
        output_path: &Path,
        context: HashMap<String, Value>,
        _params: &Self::Params,
    ) -> Result<()> {
        // 默认实现：处理嵌入式模板
        template_processor.process_embedded_template_directory(template_path, output_path, context)
    }
}

/// 项目级别生成器trait
pub trait ProjectGenerator: Generator {
    /// 生成许可证文件
    fn generate_license(&mut self, params: &Self::Params, output_path: &Path) -> Result<()>;

    /// 初始化Git仓库
    fn init_git_repository(&mut self, output_path: &Path) -> Result<()>;

    /// 安装 pre-commit hooks
    fn install_precommit(&mut self, output_path: &Path) -> Result<()>;
}
