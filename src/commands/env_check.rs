//! `NewCommand` 的环境预检查 —— 按所选语言/框架探测 Git/Go/uv/Cargo/Node/pnpm。
//!
//! 从 `prompts.rs` 拆出：交互式选择与配置仍在 `prompts.rs`，本文件只负责
//! 在生成前验证所需工具链可用。

use anyhow::Result;

use super::new::NewCommand;
use crate::constants::Language;
use crate::utils::env_checker::EnvironmentChecker;

impl NewCommand {
    pub(super) async fn check_environment(&self, language: &Language) -> Result<()> {
        tracing::info!("Checking environment...");

        let env_checker = EnvironmentChecker::new();

        // 检查Git
        if !env_checker.check_git().await? {
            return Err(anyhow::anyhow!(
                "Git is not available. Please install Git first."
            ));
        }
        tracing::info!("  Git: Available");

        // 根据语言检查相应的环境
        match language {
            Language::Go => match env_checker.check_go().await {
                Ok(true) => tracing::info!("  Go: Available"),
                Ok(false) => {
                    return Err(anyhow::anyhow!(
                        "Go is not available. Please install Go first."
                    ));
                }
                Err(e) => return Err(anyhow::anyhow!("Go version check failed: {e}")),
            },
            Language::Python => {
                match env_checker.check_uv().await {
                    Ok(true) => tracing::info!("  uv: Available"),
                    Ok(false) => {
                        return Err(anyhow::anyhow!(
                            "uv is not available. Please install uv first: https://docs.astral.sh/uv/"
                        ));
                    }
                    Err(e) => return Err(anyhow::anyhow!("uv check failed: {e}")),
                }

                // Check Python version meets minimum requirement (>= 3.12)
                match env_checker.check_python_version().await {
                    Ok(true) => tracing::info!("  Python: >= 3.12"),
                    Ok(false) => {
                        return Err(anyhow::anyhow!(
                            "Python version does not meet the minimum requirement (>= 3.12)"
                        ));
                    }
                    Err(e) => return Err(anyhow::anyhow!("Python version check failed: {e}")),
                }
            }
            Language::Rust => {
                // 检查 Cargo
                match env_checker.check_cargo().await {
                    Ok(true) => tracing::info!("  Cargo: Available"),
                    Ok(false) => {
                        return Err(anyhow::anyhow!(
                            "Cargo is not available. Please install Rust first: https://rustup.rs/"
                        ));
                    }
                    Err(e) => return Err(anyhow::anyhow!("Cargo check failed: {e}")),
                }

                // 如果选择了 Tauri 框架，还需要检查 pnpm
                if self.framework.as_ref().map(|f| f.to_lowercase()) == Some("tauri".to_string()) {
                    match env_checker.check_pnpm().await {
                        Ok(true) => tracing::info!("  pnpm: Available"),
                        Ok(false) => {
                            return Err(anyhow::anyhow!(
                                "pnpm is not available. Please install pnpm first:\n  npm install -g pnpm\n  or visit: https://pnpm.io/installation"
                            ));
                        }
                        Err(e) => return Err(anyhow::anyhow!("pnpm check failed: {e}")),
                    }
                }
            }
            Language::TypeScript => {
                // 检查 Node.js
                match env_checker.check_node().await {
                    Ok(true) => tracing::info!("  Node.js: Available"),
                    Ok(false) => {
                        return Err(anyhow::anyhow!(
                            "Node.js is not available. Please install Node.js first: https://nodejs.org/"
                        ));
                    }
                    Err(e) => return Err(anyhow::anyhow!("Node.js check failed: {e}")),
                }

                // 检查 pnpm
                match env_checker.check_pnpm().await {
                    Ok(true) => tracing::info!("  pnpm: Available"),
                    Ok(false) => {
                        return Err(anyhow::anyhow!(
                            "pnpm is not available. Please install pnpm first:\n  npm install -g pnpm\n  or visit: https://pnpm.io/installation"
                        ));
                    }
                    Err(e) => return Err(anyhow::anyhow!("pnpm check failed: {e}")),
                }
            }
        }

        Ok(())
    }
}
