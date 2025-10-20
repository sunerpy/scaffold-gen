use serde::{Deserialize, Serialize};

use crate::generators::core::{BaseParams, InheritableParams};
use crate::generators::language::go::GoParams;
use crate::generators::project::ProjectParams;

/// Gin框架参数 - 现在继承自BaseParams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GinParams {
    /// 基础参数
    pub base: BaseParams,
    /// 项目级别参数
    pub project: ProjectParams,
    /// Go语言参数
    pub go: GoParams,
}

impl Default for GinParams {
    fn default() -> Self {
        let base = BaseParams {
            default_host: Some("127.0.0.1".to_string()),
            default_port: Some(8080),
            enable_swagger: true,
            enable_cors: true,
            enable_middleware: true,
            enable_logging: true,
            ..Default::default()
        };

        Self {
            base,
            project: ProjectParams::default(),
            go: GoParams::default(),
        }
    }
}

impl InheritableParams for GinParams {
    fn base_params(&self) -> &BaseParams {
        &self.base
    }

    fn base_params_mut(&mut self) -> &mut BaseParams {
        &mut self.base
    }

    fn from_base(base: BaseParams) -> Self {
        Self {
            base,
            project: ProjectParams::default(),
            go: GoParams::default(),
        }
    }

    // Gin参数有额外的project和go参数
}

impl GinParams {
    /// 创建新的Gin参数
    pub fn new() -> Self {
        Self::default()
    }

    /// 从项目名称创建
    pub fn from_project_name(project_name: String) -> Self {
        let mut base = BaseParams::new(project_name.clone());
        // 设置Gin特定的默认值
        base.default_host = Some("127.0.0.1".to_string());
        base.default_port = Some(8080);
        base.enable_swagger = true;
        base.enable_cors = true;
        base.enable_middleware = true;
        base.enable_logging = true;

        Self {
            base,
            project: ProjectParams::from_project_name(project_name.clone()),
            go: GoParams::from_project_name(project_name),
        }
    }

    /// 设置服务器配置
    pub fn with_server(mut self, host: String, port: u16) -> Self {
        self.base.host = Some(host);
        self.base.port = Some(port);
        self
    }

    /// 设置主机地址
    pub fn with_host(mut self, host: String) -> Self {
        self.base.host = Some(host);
        self
    }

    /// 设置端口
    pub fn with_port(mut self, port: u16) -> Self {
        self.base.port = Some(port);
        self
    }

    /// 设置是否启用Swagger
    pub fn with_swagger(mut self, enable_swagger: bool) -> Self {
        self.base.enable_swagger = enable_swagger;
        self
    }

    /// 设置是否启用CORS
    pub fn with_cors(mut self, enable_cors: bool) -> Self {
        self.base.enable_cors = enable_cors;
        self
    }

    /// 设置是否启用中间件
    pub fn with_middleware(mut self, enable_middleware: bool) -> Self {
        self.base.enable_middleware = enable_middleware;
        self
    }

    /// 设置是否启用日志
    pub fn with_logging(mut self, enable_logging: bool) -> Self {
        self.base.enable_logging = enable_logging;
        self
    }

    /// 设置项目参数
    pub fn with_project(mut self, project: ProjectParams) -> Self {
        self.project = project;
        self
    }

    /// 设置Go参数
    pub fn with_go(mut self, go: GoParams) -> Self {
        self.go = go;
        self
    }

    /// 设置数据库类型
    pub fn with_database(mut self, db_type: String) -> Self {
        self.base.database_type = Some(db_type);
        self.base.enable_database = true;
        self
    }

    /// 设置是否启用Redis
    pub fn with_redis(mut self, enable_redis: bool) -> Self {
        self.base.enable_redis = enable_redis;
        self
    }

    /// 设置是否启用JWT
    pub fn with_jwt(mut self, enable_jwt: bool) -> Self {
        self.base.enable_jwt = enable_jwt;
        self
    }

    /// 设置是否启用pre-commit
    pub fn with_precommit(mut self, enable_precommit: bool) -> Self {
        self.base.enable_precommit = enable_precommit;
        self
    }

    // 为了向后兼容，提供访问器方法
    pub fn host(&self) -> Option<&String> {
        self.base.host.as_ref()
    }

    pub fn port(&self) -> Option<u16> {
        self.base.port
    }

    pub fn enable_swagger(&self) -> bool {
        self.base.enable_swagger
    }

    pub fn enable_cors(&self) -> bool {
        self.base.enable_cors
    }

    pub fn enable_middleware(&self) -> bool {
        self.base.enable_middleware
    }

    pub fn enable_logging(&self) -> bool {
        self.base.enable_logging
    }

    pub fn enable_jwt(&self) -> bool {
        self.base.enable_jwt
    }

    pub fn enable_precommit(&self) -> bool {
        self.base.enable_precommit
    }
}
