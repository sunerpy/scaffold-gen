use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// 参数系统基础trait
pub trait Parameters: Clone + Default + Send + Sync {
    /// 验证参数的有效性
    fn validate(&self) -> Result<()>;

    /// 转换为模板上下文
    fn to_template_context(&self) -> HashMap<String, Value>;

    /// 合并其他参数（可选实现）
    #[allow(dead_code)]
    fn merge(&mut self, _other: Self) -> Result<()> {
        Ok(())
    }

    /// 从环境变量覆盖参数（可选实现）
    #[allow(dead_code)]
    fn override_from_env(&mut self) -> Result<()> {
        Ok(())
    }
}

/// 参数构建器trait - 用于链式构建参数
#[allow(dead_code)]
pub trait ParameterBuilder<T: Parameters> {
    /// 构建最终的参数对象
    fn build(self) -> Result<T>;
}

/// 通用参数验证辅助函数
pub mod validation {
    use anyhow::{Result, anyhow};

    /// 验证项目名称
    #[allow(dead_code)]
    pub fn validate_project_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(anyhow!("Project name cannot be empty"));
        }

        if name.contains(' ') {
            return Err(anyhow!("Project name cannot contain spaces"));
        }

        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(anyhow!(
                "Project name can only contain alphanumeric characters, hyphens, and underscores"
            ));
        }

        Ok(())
    }

    /// 验证端口号
    #[allow(dead_code)]
    pub fn validate_port(port: u16) -> Result<()> {
        if port < 1024 {
            return Err(anyhow!("Port number should be >= 1024"));
        }
        Ok(())
    }

    /// 验证主机地址
    #[allow(dead_code)]
    pub fn validate_host(host: &str) -> Result<()> {
        if host.is_empty() {
            return Err(anyhow!("Host cannot be empty"));
        }
        Ok(())
    }
}
