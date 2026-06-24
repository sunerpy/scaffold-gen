use anyhow::{Context, Result};
use std::path::Path;

use super::parameters::Vue3Params;
use crate::generators::core::Generator;
use crate::utils::toolchain::{self, ExternalCommand};

/// Vue3框架级别生成器实现
#[derive(Debug)]
pub struct Vue3Generator {}

impl Vue3Generator {
    /// 检查 pnpm 是否已安装
    pub fn check_pnpm() -> Result<bool> {
        Ok(toolchain::tool_available("pnpm"))
    }

    /// 使用 pnpm create vue 创建项目
    pub fn create_vue3_project(project_name: &str, output_path: &Path) -> Result<()> {
        tracing::debug!("🚀 Creating Vue3 project with create-vue...");

        // 获取父目录
        let parent_dir = output_path.parent().unwrap_or_else(|| Path::new("."));

        // 使用 pnpm create vue 创建项目
        // 使用非交互模式，指定所有选项
        let outcome = ExternalCommand::new("pnpm")
            .args([
                "create",
                "vue@latest",
                project_name,
                "--typescript",
                "--router",
                "--pinia",
                "--eslint",
                "--prettier",
            ])
            .current_dir(parent_dir)
            .run()
            .context("Failed to execute pnpm create vue")?;

        if outcome.success() {
            tracing::debug!("✅ Vue3 project created successfully");
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to create Vue3 project:\nstdout: {}\nstderr: {}",
                outcome.stdout(),
                outcome.stderr()
            ))
        }
    }

    /// 安装 Tailwind CSS
    pub fn install_tailwind(output_path: &Path) -> Result<()> {
        tracing::debug!("📦 Installing Tailwind CSS...");

        // 安装 Tailwind CSS 依赖
        let outcome = ExternalCommand::new("pnpm")
            .args([
                "add",
                "-D",
                "tailwindcss",
                "postcss",
                "autoprefixer",
                "@tailwindcss/forms",
                "@tailwindcss/typography",
            ])
            .current_dir(output_path)
            .run()
            .context("Failed to install Tailwind CSS")?;

        if !outcome.success() {
            tracing::warn!(
                "⚠️ Warning: Failed to install Tailwind CSS: {}",
                outcome.stderr()
            );
        }

        // 初始化 Tailwind CSS
        let outcome = ExternalCommand::new("pnpm")
            .args(["exec", "tailwindcss", "init", "-p"])
            .current_dir(output_path)
            .run()
            .context("Failed to initialize Tailwind CSS")?;

        if outcome.success() {
            tracing::debug!("✅ Tailwind CSS installed successfully");
        } else {
            tracing::warn!(
                "⚠️ Warning: Failed to initialize Tailwind CSS: {}",
                outcome.stderr()
            );
        }

        Ok(())
    }

    /// 安装前端依赖
    pub fn install_dependencies(output_path: &Path) -> Result<()> {
        tracing::debug!("📦 Installing frontend dependencies...");

        let outcome = ExternalCommand::new("pnpm")
            .arg("install")
            .current_dir(output_path)
            .run()
            .context("Failed to execute pnpm install")?;

        if outcome.success() {
            tracing::debug!("✅ Dependencies installed successfully");
        } else {
            tracing::warn!(
                "⚠️ Warning: Failed to install dependencies: {}",
                outcome.stderr()
            );
            // 不返回错误，让用户手动安装
        }

        Ok(())
    }
}

impl Generator for Vue3Generator {
    type Params = Vue3Params;

    fn name(&self) -> &'static str {
        "Vue3"
    }

    fn get_template_path(&self) -> &'static str {
        "frameworks/typescript/vue3"
    }
}
