use anyhow::{Result, anyhow};
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
            let major: u32 = captures.get(1).unwrap().as_str().parse()?;
            let minor: u32 = captures.get(2).unwrap().as_str().parse()?;

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

    /// 获取Go版本字符串（用于模板参数）
    #[allow(dead_code)]
    pub async fn get_go_version(&self) -> Result<String> {
        let output = Command::new("go").arg("version").output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get Go version"));
        }

        let version_str = String::from_utf8_lossy(&output.stdout);
        let re = Regex::new(r"go(\d+)\.(\d+)(?:\.(\d+))?")?;

        if let Some(captures) = re.captures(&version_str) {
            let major = captures.get(1).unwrap().as_str();
            let minor = captures.get(2).unwrap().as_str();

            // 返回格式化的版本字符串，如 "1.25"
            Ok(format!("{major}.{minor}"))
        } else {
            Err(anyhow!("Unable to parse Go version"))
        }
    }

    /// 检查 Node.js 是否可用
    #[allow(dead_code)]
    pub async fn check_node(&self) -> Result<bool> {
        match which("node") {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// 检查 Rust 是否可用并验证版本
    #[allow(dead_code)]
    pub async fn check_rust(&self) -> Result<bool> {
        match which("cargo") {
            Ok(_) => {
                // 检查Rust版本是否满足要求 (>= 1.88)
                self.check_rust_version().await
            }
            Err(_) => Ok(false),
        }
    }

    /// 检查Rust版本是否满足要求
    #[allow(dead_code)]
    async fn check_rust_version(&self) -> Result<bool> {
        let output = Command::new("rustc").arg("--version").output()?;

        if !output.status.success() {
            return Ok(false);
        }

        let version_str = String::from_utf8_lossy(&output.stdout);
        let re = Regex::new(r"rustc (\d+)\.(\d+)\.(\d+)")?;

        if let Some(captures) = re.captures(&version_str) {
            let major: u32 = captures.get(1).unwrap().as_str().parse()?;
            let minor: u32 = captures.get(2).unwrap().as_str().parse()?;

            // 要求Rust版本 >= 1.88
            if major > 1 || (major == 1 && minor >= 88) {
                Ok(true)
            } else {
                Err(anyhow!(
                    "Rust version {major}.{minor} is not supported. Minimum required version is 1.88"
                ))
            }
        } else {
            Err(anyhow!("Unable to parse Rust version"))
        }
    }

    /// 检查 Python 是否可用并验证版本和uv工具
    #[allow(dead_code)]
    pub async fn check_python(&self) -> Result<bool> {
        // 首先检查Python版本
        let python_ok = self.check_python_version().await?;
        if !python_ok {
            return Ok(false);
        }

        // 然后检查uv工具
        self.check_uv().await
    }

    /// 检查Python版本是否满足要求
    #[allow(dead_code)]
    async fn check_python_version(&self) -> Result<bool> {
        let output = Command::new("python").arg("--version").output()?;

        if !output.status.success() {
            return Ok(false);
        }

        let version_str = String::from_utf8_lossy(&output.stdout);
        let re = Regex::new(r"Python (\d+)\.(\d+)\.(\d+)")?;

        if let Some(captures) = re.captures(&version_str) {
            let major: u32 = captures.get(1).unwrap().as_str().parse()?;
            let minor: u32 = captures.get(2).unwrap().as_str().parse()?;

            // 要求Python版本 >= 3.12
            if major > 3 || (major == 3 && minor >= 12) {
                Ok(true)
            } else {
                Err(anyhow!(
                    "Python version {major}.{minor} is not supported. Minimum required version is 3.12"
                ))
            }
        } else {
            Err(anyhow!("Unable to parse Python version"))
        }
    }

    /// 检查uv工具是否可用
    #[allow(dead_code)]
    async fn check_uv(&self) -> Result<bool> {
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
}
