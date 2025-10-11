use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::generators::core::{Parameters, validation};
use crate::generators::language::go::parameters::GoParams;
use crate::generators::project::parameters::ProjectParams;

/// Gin框架级别参数 - 继承项目级别和语言级别参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GinParams {
    /// 项目级别参数
    pub project: ProjectParams,
    /// Go语言级别参数
    pub go: GoParams,
    /// 服务器主机地址
    pub host: String,
    /// 服务器端口
    pub port: u16,
    /// 是否启用Swagger文档
    pub enable_swagger: bool,
    /// 是否启用CORS
    pub enable_cors: bool,
    /// 是否启用日志中间件
    pub enable_logging: bool,
    /// 是否启用恢复中间件
    pub enable_recovery: bool,
    /// 是否启用限流中间件
    pub enable_rate_limit: bool,
    /// 是否启用JWT认证
    pub enable_jwt: bool,
    /// 是否启用数据库支持
    pub enable_database: bool,
    /// 数据库类型
    pub database_type: Option<String>,
    /// 是否启用Redis
    pub enable_redis: bool,
    /// 是否启用pre-commit hooks
    pub enable_precommit: bool,
}

impl Default for GinParams {
    fn default() -> Self {
        Self {
            project: ProjectParams::default(),
            go: GoParams::default(),
            host: "localhost".to_string(),
            port: 8080,
            enable_swagger: true,
            enable_cors: true,
            enable_logging: true,
            enable_recovery: true,
            enable_rate_limit: false,
            enable_jwt: false,
            enable_database: false,
            database_type: None,
            enable_redis: false,
            enable_precommit: true,
        }
    }
}

impl Parameters for GinParams {
    fn validate(&self) -> Result<()> {
        // 验证继承的参数
        self.project.validate()?;
        self.go.validate()?;

        // 验证框架特定参数
        validation::validate_host(&self.host)?;
        validation::validate_port(self.port)?;

        if self.enable_database && self.database_type.is_none() {
            return Err(anyhow::anyhow!(
                "Database type must be specified when database is enabled"
            ));
        }

        Ok(())
    }

    fn to_template_context(&self) -> HashMap<String, Value> {
        let mut context = HashMap::new();

        // 合并项目级别参数
        let project_context = self.project.to_template_context();
        context.extend(project_context);

        // 合并Go语言级别参数
        let go_context = self.go.to_template_context();
        context.extend(go_context);

        // 添加框架特定参数
        context.insert("host".to_string(), json!(self.host));
        context.insert("port".to_string(), json!(self.port));
        context.insert("enable_swagger".to_string(), json!(self.enable_swagger));
        context.insert("enable_cors".to_string(), json!(self.enable_cors));
        context.insert("enable_logging".to_string(), json!(self.enable_logging));
        context.insert("enable_recovery".to_string(), json!(self.enable_recovery));
        context.insert(
            "enable_rate_limit".to_string(),
            json!(self.enable_rate_limit),
        );
        context.insert("enable_jwt".to_string(), json!(self.enable_jwt));
        context.insert("enable_database".to_string(), json!(self.enable_database));
        context.insert("enable_redis".to_string(), json!(self.enable_redis));
        context.insert("enable_precommit".to_string(), json!(self.enable_precommit));

        if let Some(ref db_type) = self.database_type {
            context.insert("database_type".to_string(), json!(db_type));
        }

        // 生成服务器地址
        context.insert(
            "server_addr".to_string(),
            json!(format!("{}:{}", self.host, self.port)),
        );

        context
    }

    fn override_from_env(&mut self) -> Result<()> {
        // 继承的参数也从环境变量覆盖
        self.project.override_from_env()?;
        self.go.override_from_env()?;

        // 框架特定的环境变量覆盖
        if let Ok(host) = std::env::var("SERVER_HOST") {
            self.host = host;
        }

        if let Ok(port_str) = std::env::var("SERVER_PORT") {
            if let Ok(port) = port_str.parse::<u16>() {
                self.port = port;
            }
        }

        if let Ok(db_type) = std::env::var("DATABASE_TYPE") {
            self.database_type = Some(db_type);
            self.enable_database = true;
        }

        Ok(())
    }
}

impl GinParams {
    /// 创建新的Gin参数
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// 从项目参数和Go参数创建
    pub fn from_project_and_go(project: ProjectParams, go: GoParams) -> Self {
        Self {
            project,
            go,
            ..Default::default()
        }
    }

    /// 设置项目参数
    #[allow(dead_code)]
    pub fn with_project(mut self, project: ProjectParams) -> Self {
        self.project = project;
        self
    }

    /// 设置Go参数
    #[allow(dead_code)]
    pub fn with_go(mut self, go: GoParams) -> Self {
        self.go = go;
        self
    }

    /// 设置服务器配置
    pub fn with_server(mut self, host: String, port: u16) -> Self {
        self.host = host;
        self.port = port;
        self
    }

    /// 启用Swagger
    pub fn with_swagger(mut self, enable: bool) -> Self {
        self.enable_swagger = enable;
        self
    }

    /// 启用CORS
    pub fn with_cors(mut self, enable: bool) -> Self {
        self.enable_cors = enable;
        self
    }

    /// 启用JWT认证
    pub fn with_jwt(mut self, enable: bool) -> Self {
        self.enable_jwt = enable;
        self
    }

    /// 启用数据库支持
    pub fn with_database(mut self, db_type: String) -> Self {
        self.enable_database = true;
        self.database_type = Some(db_type);
        self
    }

    /// 启用Redis
    pub fn with_redis(mut self, enable: bool) -> Self {
        self.enable_redis = enable;
        self
    }

    /// 启用限流
    #[allow(dead_code)]
    pub fn with_rate_limit(mut self, enable: bool) -> Self {
        self.enable_rate_limit = enable;
        self
    }

    /// 启用pre-commit hooks
    pub fn with_precommit(mut self, enable: bool) -> Self {
        self.enable_precommit = enable;
        self
    }
}
