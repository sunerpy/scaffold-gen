use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Go 工具集，提供常用的 Go 命令封装
pub struct GoTools;

impl GoTools {
    /// 运行 go mod tidy 命令
    pub fn mod_tidy(output_path: &Path) -> Result<()> {
        println!("Running go mod tidy...");

        let status = Command::new("go")
            .args(["mod", "tidy"])
            .current_dir(output_path)
            .status()
            .context("Failed to execute go mod tidy command")?;

        if status.success() {
            println!("Dependencies organized with go mod tidy");
        } else {
            println!("Warning: Failed to run go mod tidy, you may need to run it manually");
        }

        Ok(())
    }

    /// 运行 go mod init 命令
    #[allow(dead_code)]
    pub fn mod_init(output_path: &Path, module_name: &str) -> Result<()> {
        println!("Initializing Go module: {module_name}");

        let status = Command::new("go")
            .args(["mod", "init", module_name])
            .current_dir(output_path)
            .status()
            .context("Failed to execute go mod init command")?;

        if status.success() {
            println!("Go module initialized: {module_name}");
        } else {
            return Err(anyhow::anyhow!("Failed to initialize Go module"));
        }

        Ok(())
    }

    /// 检查 Go 是否已安装
    #[allow(dead_code)]
    pub fn check_installation() -> Result<String> {
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

    /// 运行 go get 命令安装依赖
    #[allow(dead_code)]
    pub fn get_dependency(output_path: &Path, dependency: &str) -> Result<()> {
        println!("Installing Go dependency: {dependency}");

        let status = Command::new("go")
            .args(["get", dependency])
            .current_dir(output_path)
            .status()
            .context("Failed to execute go get command")?;

        if status.success() {
            println!("Dependency installed: {dependency}");
        } else {
            println!("Warning: Failed to install dependency: {dependency}");
        }

        Ok(())
    }
}
