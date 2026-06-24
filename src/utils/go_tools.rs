use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Go 工具集，提供常用的 Go 命令封装
pub struct GoTools;

impl GoTools {
    /// 运行 go mod tidy 命令
    pub fn mod_tidy(output_path: &Path) -> Result<()> {
        tracing::debug!("Running go mod tidy...");

        let status = Command::new("go")
            .args(["mod", "tidy"])
            .current_dir(output_path)
            .status()
            .context("Failed to execute go mod tidy command")?;

        if status.success() {
            tracing::debug!("Dependencies organized with go mod tidy");
        } else {
            tracing::warn!("Warning: Failed to run go mod tidy, you may need to run it manually");
        }

        Ok(())
    }
}
