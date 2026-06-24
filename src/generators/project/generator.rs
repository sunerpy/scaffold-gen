use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use super::parameters::ProjectParams;
use crate::generators::core::{Generator, Parameters, ProjectGenerator, TemplateProcessor};

/// 项目级别生成器实现
pub struct ProjectScaffolder {
    template_processor: TemplateProcessor,
}

impl ProjectScaffolder {
    /// 创建新的项目生成器
    pub fn new() -> Result<Self> {
        Ok(Self {
            template_processor: TemplateProcessor::new()?,
        })
    }

    /// 获取Git作者信息
    fn get_git_author(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["config", "--global", "user.name"])
            .output()
            .context("Failed to get git author name")?;

        if output.status.success() {
            let author = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !author.is_empty() {
                return Ok(author);
            }
        }

        // 如果Git配置不存在，返回默认值
        Ok("Unknown".to_string())
    }
}

impl Generator for ProjectScaffolder {
    type Params = ProjectParams;

    fn name(&self) -> &'static str {
        "project"
    }

    fn get_template_path(&self) -> &'static str {
        "project"
    }

    fn generate(&mut self, params: Self::Params, output_path: &Path) -> Result<()> {
        params.validate()?;

        // 生成LICENSE文件
        self.generate_license(&params, output_path)?;

        // 初始化Git仓库
        if params.enable_git() {
            self.init_git_repository(output_path)?;
        }

        // 安装 pre-commit hooks
        if params.enable_precommit() {
            self.install_precommit(output_path)?;
        }

        Ok(())
    }
}

impl ProjectGenerator for ProjectScaffolder {
    fn generate_license(&mut self, params: &Self::Params, output_path: &Path) -> Result<()> {
        let license_template = format!("licenses/{}.tmpl", params.license());

        if !self.template_processor.template_exists(&license_template) {
            return Err(anyhow::anyhow!(
                "License template not found: {}",
                params.license()
            ));
        }

        let template_path = self
            .template_processor
            .get_template_path(&license_template)
            .context("Failed to get license template path")?;

        let license_file = output_path.join("LICENSE");
        let mut context = params.to_template_context();

        // 如果参数中没有作者信息，尝试从Git获取
        if params.author().is_none()
            && let Ok(git_author) = self.get_git_author()
        {
            context.insert("author".to_string(), serde_json::json!(git_author));
        }

        let mut template_processor =
            TemplateProcessor::new().context("Failed to create template processor")?;

        template_processor
            .process_template_file(&template_path, &license_file, context)
            .context("Failed to generate LICENSE file")?;

        Ok(())
    }

    fn init_git_repository(&mut self, output_path: &Path) -> Result<()> {
        let status = Command::new("git")
            .args(["init"])
            .current_dir(output_path)
            .status();

        match status {
            Ok(status) if status.success() => {
                tracing::info!("Initialized Git repository");
                Ok(())
            }
            _ => {
                tracing::warn!("⚠️  Warning: Failed to initialize Git repository");
                Ok(())
            }
        }
    }

    fn install_precommit(&mut self, output_path: &Path) -> Result<()> {
        // 检查是否存在 .pre-commit-config.yaml 文件
        let precommit_config = output_path.join(".pre-commit-config.yaml");
        if !precommit_config.exists() {
            tracing::debug!("No .pre-commit-config.yaml found, skipping pre-commit installation");
            return Ok(());
        }

        // 尝试安装 pre-commit hooks
        let status = Command::new("pre-commit")
            .args(["install"])
            .current_dir(output_path)
            .status();

        match status {
            Ok(status) if status.success() => {
                tracing::info!("Pre-commit hooks installed");
            }
            _ => {
                tracing::warn!(
                    "⚠️  Warning: Failed to install pre-commit hooks, you may need to install them manually"
                );
                tracing::warn!("   Run: pre-commit install");
            }
        }

        Ok(())
    }
}
