//! Gin 项目生成选项 —— `GinProjectOptions` 数据结构与其链式 builder。
//!
//! 从 `orchestrator.rs` 拆出：仅承载 Gin 生成所需的项目级/语言级/框架级开关，
//! 由 `new.rs` 填充、`GeneratorOrchestrator::generate_gin_project` 消费。

/// Gin项目生成选项
#[derive(Debug, Default, Clone)]
pub struct GinProjectOptions {
    // 项目级别选项
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub enable_git: Option<bool>,

    // 语言级别选项 (Go)
    pub go_version: Option<String>,
    pub module_name: Option<String>,

    // 框架级别选项 (Gin)
    pub host: Option<String>,
    pub port: Option<u16>,
    pub enable_swagger: Option<bool>,
    pub enable_cors: Option<bool>,
    pub enable_jwt: Option<bool>,
    pub enable_precommit: Option<bool>,
    pub enable_redis: Option<bool>,
    pub database_type: Option<String>,
}

impl GinProjectOptions {
    /// 创建新的选项
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置许可证
    pub fn with_license(mut self, license: String) -> Self {
        self.license = Some(license);
        self
    }

    /// 设置服务器配置
    pub fn with_server(mut self, host: String, port: u16) -> Self {
        self.host = Some(host);
        self.port = Some(port);
        self
    }

    /// 启用Swagger
    pub fn with_swagger(mut self, enable: bool) -> Self {
        self.enable_swagger = Some(enable);
        self
    }

    /// 启用pre-commit
    pub fn with_precommit(mut self, enable: bool) -> Self {
        self.enable_precommit = Some(enable);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_default_have_all_none() {
        for opts in [GinProjectOptions::new(), GinProjectOptions::default()] {
            assert!(opts.description.is_none());
            assert!(opts.author.is_none());
            assert!(opts.license.is_none());
            assert!(opts.enable_git.is_none());
            assert!(opts.go_version.is_none());
            assert!(opts.module_name.is_none());
            assert!(opts.host.is_none());
            assert!(opts.port.is_none());
            assert!(opts.enable_swagger.is_none());
            assert!(opts.enable_cors.is_none());
            assert!(opts.enable_jwt.is_none());
            assert!(opts.enable_precommit.is_none());
            assert!(opts.enable_redis.is_none());
            assert!(opts.database_type.is_none());
        }
    }

    #[test]
    fn with_license_sets_license() {
        let opts = GinProjectOptions::new().with_license("Apache-2.0".to_string());
        assert_eq!(opts.license, Some("Apache-2.0".to_string()));
        assert!(opts.host.is_none());
    }

    #[test]
    fn with_server_sets_host_and_port() {
        let opts = GinProjectOptions::new().with_server("0.0.0.0".to_string(), 9090);
        assert_eq!(opts.host, Some("0.0.0.0".to_string()));
        assert_eq!(opts.port, Some(9090));
    }

    #[test]
    fn with_swagger_toggles_flag() {
        assert_eq!(
            GinProjectOptions::new().with_swagger(true).enable_swagger,
            Some(true)
        );
        assert_eq!(
            GinProjectOptions::new().with_swagger(false).enable_swagger,
            Some(false)
        );
    }

    #[test]
    fn with_precommit_toggles_flag() {
        assert_eq!(
            GinProjectOptions::new()
                .with_precommit(true)
                .enable_precommit,
            Some(true)
        );
        assert_eq!(
            GinProjectOptions::new()
                .with_precommit(false)
                .enable_precommit,
            Some(false)
        );
    }

    #[test]
    fn builders_chain_and_compose() {
        let opts = GinProjectOptions::new()
            .with_license("MIT".to_string())
            .with_server("127.0.0.1".to_string(), 8080)
            .with_swagger(true)
            .with_precommit(false);

        assert_eq!(opts.license, Some("MIT".to_string()));
        assert_eq!(opts.host, Some("127.0.0.1".to_string()));
        assert_eq!(opts.port, Some(8080));
        assert_eq!(opts.enable_swagger, Some(true));
        assert_eq!(opts.enable_precommit, Some(false));
    }
}
