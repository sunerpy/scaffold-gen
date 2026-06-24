use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use super::parameters::Vue3Params;
use crate::generators::core::Generator;

/// Vue3框架级别生成器实现
#[derive(Debug)]
pub struct Vue3Generator {}

impl Vue3Generator {
    /// 创建新的Vue3生成器
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// 检查 pnpm 是否已安装
    pub fn check_pnpm() -> Result<bool> {
        match Command::new("pnpm").arg("--version").output() {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    /// 使用 pnpm create vue 创建项目
    pub fn create_vue3_project(project_name: &str, output_path: &Path) -> Result<()> {
        println!("🚀 Creating Vue3 project with create-vue...");

        // 获取父目录
        let parent_dir = output_path.parent().unwrap_or_else(|| Path::new("."));

        // 使用 pnpm create vue 创建项目
        // 使用非交互模式，指定所有选项
        let output = Command::new("pnpm")
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
            .output()
            .context("Failed to execute pnpm create vue")?;

        if output.status.success() {
            println!("✅ Vue3 project created successfully");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(anyhow::anyhow!(
                "Failed to create Vue3 project:\nstdout: {stdout}\nstderr: {stderr}"
            ))
        }
    }

    /// 安装 Tailwind CSS
    pub fn install_tailwind(output_path: &Path) -> Result<()> {
        println!("📦 Installing Tailwind CSS...");

        // 安装 Tailwind CSS 依赖
        let output = Command::new("pnpm")
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
            .output()
            .context("Failed to install Tailwind CSS")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("⚠️ Warning: Failed to install Tailwind CSS: {stderr}");
        }

        // 初始化 Tailwind CSS
        let output = Command::new("pnpm")
            .args(["exec", "tailwindcss", "init", "-p"])
            .current_dir(output_path)
            .output()
            .context("Failed to initialize Tailwind CSS")?;

        if output.status.success() {
            println!("✅ Tailwind CSS installed successfully");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("⚠️ Warning: Failed to initialize Tailwind CSS: {stderr}");
        }

        Ok(())
    }

    /// 安装前端依赖
    pub fn install_dependencies(output_path: &Path) -> Result<()> {
        println!("📦 Installing frontend dependencies...");

        let output = Command::new("pnpm")
            .arg("install")
            .current_dir(output_path)
            .output()
            .context("Failed to execute pnpm install")?;

        if output.status.success() {
            println!("✅ Dependencies installed successfully");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("⚠️ Warning: Failed to install dependencies: {stderr}");
            // 不返回错误，让用户手动安装
            Ok(())
        }
    }
}

impl Default for Vue3Generator {
    fn default() -> Self {
        Self::new().expect("Failed to create Vue3Generator")
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
