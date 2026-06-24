use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use super::parameters::ReactParams;
use crate::generators::core::Generator;

/// React框架级别生成器实现
#[derive(Debug)]
pub struct ReactGenerator {}

impl ReactGenerator {
    /// 检查 pnpm 是否已安装
    pub fn check_pnpm() -> Result<bool> {
        match Command::new("pnpm").arg("--version").output() {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    /// 使用 pnpm create vite 创建 React 项目
    pub fn create_react_project(project_name: &str, output_path: &Path) -> Result<()> {
        println!("🚀 Creating React project with Vite...");

        // 获取父目录
        let parent_dir = output_path.parent().unwrap_or_else(|| Path::new("."));

        // 使用 pnpm create vite 创建项目
        let output = Command::new("pnpm")
            .args([
                "create",
                "vite@latest",
                project_name,
                "--template",
                "react-ts",
            ])
            .current_dir(parent_dir)
            .output()
            .context("Failed to execute pnpm create vite")?;

        if output.status.success() {
            println!("✅ React project created successfully");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(anyhow::anyhow!(
                "Failed to create React project:\nstdout: {stdout}\nstderr: {stderr}"
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

    /// 安装 React Router
    pub fn install_router(output_path: &Path) -> Result<()> {
        println!("📦 Installing React Router...");

        let output = Command::new("pnpm")
            .args(["add", "react-router-dom"])
            .current_dir(output_path)
            .output()
            .context("Failed to install React Router")?;

        if output.status.success() {
            println!("✅ React Router installed successfully");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("⚠️ Warning: Failed to install React Router: {stderr}");
        }

        Ok(())
    }

    /// 安装状态管理库
    pub fn install_state_management(output_path: &Path, state_management: &str) -> Result<()> {
        println!("📦 Installing {state_management}...");

        let packages = match state_management {
            "zustand" => vec!["zustand"],
            "redux" => vec!["@reduxjs/toolkit", "react-redux"],
            "jotai" => vec!["jotai"],
            _ => vec!["zustand"], // 默认使用 zustand
        };

        let mut args = vec!["add"];
        args.extend(packages.iter().copied());

        let output = Command::new("pnpm")
            .args(&args)
            .current_dir(output_path)
            .output()
            .context("Failed to install state management library")?;

        if output.status.success() {
            println!("✅ {state_management} installed successfully");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("⚠️ Warning: Failed to install {state_management}: {stderr}");
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

impl Generator for ReactGenerator {
    type Params = ReactParams;

    fn name(&self) -> &'static str {
        "React"
    }

    fn get_template_path(&self) -> &'static str {
        "frameworks/typescript/react"
    }
}
