use anyhow::{Context, Result, anyhow};
use regex::Regex;
use std::process::Command;
use which::which;

pub struct EnvironmentChecker;

impl Default for EnvironmentChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentChecker {
    pub fn new() -> Self {
        Self
    }

    /// 检查 Git 是否可用
    pub async fn check_git(&self) -> Result<bool> {
        match which("git") {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// 检查 Swag 是否可用
    pub async fn check_swag(&self) -> Result<bool> {
        match which("swag") {
            Ok(_) => {
                // 进一步验证swag命令是否可以正常执行
                match Command::new("swag").args(["--version"]).output() {
                    Ok(output) => Ok(output.status.success()),
                    Err(_) => Ok(false),
                }
            }
            Err(_) => Ok(false),
        }
    }

    /// 检查 Go 是否可用并验证版本
    pub async fn check_go(&self) -> Result<bool> {
        match which("go") {
            Ok(_) => {
                // 检查Go版本是否满足要求 (>= 1.24)
                self.check_go_version().await
            }
            Err(_) => Ok(false),
        }
    }

    /// 检查Go版本是否满足要求
    async fn check_go_version(&self) -> Result<bool> {
        let output = Command::new("go").arg("version").output()?;

        if !output.status.success() {
            return Ok(false);
        }

        let version_str = String::from_utf8_lossy(&output.stdout);
        let re = Regex::new(r"go(\d+)\.(\d+)(?:\.(\d+))?")?;

        if let Some(captures) = re.captures(&version_str) {
            let major: u32 = captures
                .get(1)
                .context("Go version regex missing major capture group")?
                .as_str()
                .parse()?;
            let minor: u32 = captures
                .get(2)
                .context("Go version regex missing minor capture group")?
                .as_str()
                .parse()?;

            // 要求Go版本 >= 1.24
            if major > 1 || (major == 1 && minor >= 24) {
                Ok(true)
            } else {
                Err(anyhow!(
                    "Go version {major}.{minor} is not supported. Minimum required version is 1.24"
                ))
            }
        } else {
            Err(anyhow!("Unable to parse Go version"))
        }
    }

    /// 检查 Node.js 是否可用
    pub async fn check_node(&self) -> Result<bool> {
        match which("node") {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// 获取Python版本字符串（用于模板参数）
    pub async fn get_python_version(&self) -> Result<String> {
        let output = Command::new("python").arg("--version").output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get Python version"));
        }

        let version_str = String::from_utf8_lossy(&output.stdout);
        let re = Regex::new(r"Python (\d+)\.(\d+)(?:\.(\d+))?")?;

        if let Some(captures) = re.captures(&version_str) {
            let major = captures
                .get(1)
                .context("Python version regex missing major capture group")?
                .as_str();
            let minor = captures
                .get(2)
                .context("Python version regex missing minor capture group")?
                .as_str();

            // 返回格式化的版本字符串，如 "3.12"
            Ok(format!("{major}.{minor}"))
        } else {
            Err(anyhow!("Unable to parse Python version"))
        }
    }

    /// Check if the Python version meets the minimum requirement (>= 3.12)
    pub async fn check_python_version(&self) -> Result<bool> {
        let version = self.get_python_version().await?;
        let parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();

        if parts.len() >= 2 && (parts[0] > 3 || (parts[0] == 3 && parts[1] >= 12)) {
            Ok(true)
        } else {
            Err(anyhow!(
                "Python version {version} is not supported. Minimum required: 3.12"
            ))
        }
    }

    /// 检查uv工具是否可用
    pub async fn check_uv(&self) -> Result<bool> {
        match which("uv") {
            Ok(_) => {
                let output = Command::new("uv").arg("--version").output()?;

                if output.status.success() {
                    Ok(true)
                } else {
                    Err(anyhow!("uv command is available but not working properly"))
                }
            }
            Err(_) => Err(anyhow!(
                "uv command is not available. Please install uv for Python package management"
            )),
        }
    }

    /// 获取uv版本字符串（纯版本号，剥离前缀和后缀）
    pub async fn get_uv_version(&self) -> Result<String> {
        let output = Command::new("uv").arg("--version").output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get uv version"));
        }

        let version_str = String::from_utf8_lossy(&output.stdout);
        let trimmed = version_str.trim();

        // parse_uv_version handles extraction of pure version token
        parse_uv_version(trimmed)
            .ok_or_else(|| anyhow!("Unable to parse uv version from: {trimmed}"))
    }

    /// 检查 Cargo 是否可用
    pub async fn check_cargo(&self) -> Result<bool> {
        match which("cargo") {
            Ok(_) => {
                // 验证cargo命令是否可以正常执行
                match Command::new("cargo").args(["--version"]).output() {
                    Ok(output) => Ok(output.status.success()),
                    Err(_) => Ok(false),
                }
            }
            Err(_) => Ok(false),
        }
    }

    /// 获取Rust版本字符串（用于模板参数）
    pub async fn get_rust_version(&self) -> Result<String> {
        let output = Command::new("rustc").arg("--version").output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get Rust version"));
        }

        let version_str = String::from_utf8_lossy(&output.stdout);
        let re = Regex::new(r"rustc (\d+)\.(\d+)(?:\.(\d+))?")?;

        if let Some(captures) = re.captures(&version_str) {
            let major = captures
                .get(1)
                .context("Rust version regex missing major capture group")?
                .as_str();
            let minor = captures
                .get(2)
                .context("Rust version regex missing minor capture group")?
                .as_str();

            // 返回格式化的版本字符串，如 "1.75"
            Ok(format!("{major}.{minor}"))
        } else {
            Err(anyhow!("Unable to parse Rust version"))
        }
    }

    /// 检查 pnpm 是否可用
    pub async fn check_pnpm(&self) -> Result<bool> {
        match which("pnpm") {
            Ok(_) => match Command::new("pnpm").args(["--version"]).output() {
                Ok(output) => Ok(output.status.success()),
                Err(_) => Ok(false),
            },
            Err(_) => Ok(false),
        }
    }
}

/// Extract pure version token from uv --version output.
///
/// Input examples:
/// - "uv 0.9.1" → "0.9.1"
/// - "uv 0.11.8 (x86_64-unknown-linux-musl)" → "0.11.8"
///
/// Strategy: split by whitespace, take token at index 1 (after "uv").
/// Returns None if insufficient tokens.
fn parse_uv_version(raw: &str) -> Option<String> {
    raw.split_whitespace().nth(1).map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uv_version_official_release() {
        let input = "uv 0.9.1";
        assert_eq!(parse_uv_version(input), Some("0.9.1".to_string()));
    }

    #[test]
    fn test_parse_uv_version_musl_with_arch() {
        let input = "uv 0.11.8 (x86_64-unknown-linux-musl)";
        assert_eq!(parse_uv_version(input), Some("0.11.8".to_string()));
    }

    #[test]
    fn test_parse_uv_version_multi_space() {
        let input = "uv  0.12.0  (aarch64-apple-darwin)";
        assert_eq!(parse_uv_version(input), Some("0.12.0".to_string()));
    }

    #[test]
    fn test_parse_uv_version_no_prefix() {
        let input = "0.10.5";
        assert_eq!(parse_uv_version(input), None);
    }

    #[test]
    fn test_parse_uv_version_insufficient_tokens() {
        let input = "uv";
        assert_eq!(parse_uv_version(input), None);
    }

    #[test]
    fn test_parse_uv_version_empty() {
        let input = "";
        assert_eq!(parse_uv_version(input), None);
    }
}
