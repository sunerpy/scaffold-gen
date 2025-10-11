use anyhow::Result;

/// 验证项目名称
pub fn validate_project_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow::anyhow!("Project name cannot be empty"));
    }

    if name.len() > 100 {
        return Err(anyhow::anyhow!(
            "Project name is too long (max 100 characters)"
        ));
    }

    // 检查是否包含非法字符
    if name
        .chars()
        .any(|c| matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*' | '\\' | '/'))
    {
        return Err(anyhow::anyhow!("Project name contains invalid characters"));
    }

    Ok(())
}

/// 验证端口号
pub fn validate_port(port: u16) -> Result<()> {
    if port < 1024 {
        return Err(anyhow::anyhow!(
            "Port number should be >= 1024 (got {port})"
        ));
    }

    // u16 类型的最大值就是 65535，无需额外检查

    Ok(())
}

/// 验证主机地址
pub fn validate_host(host: &str) -> Result<()> {
    if host.is_empty() {
        return Err(anyhow::anyhow!("Host cannot be empty"));
    }

    // 简单的主机名验证
    if host.len() > 253 {
        return Err(anyhow::anyhow!("Host name is too long"));
    }

    Ok(())
}
