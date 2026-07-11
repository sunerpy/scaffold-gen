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

    // Gin参数有额外的project和go参数
}

impl GinParams {
    /// 创建新的Gin参数
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
    pub fn enable_swagger(&self) -> bool {
        self.base.enable_swagger
    }

    pub fn enable_precommit(&self) -> bool {
        self.base.enable_precommit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::core::Parameters;

    #[test]
    fn default_sets_gin_specific_flags() {
        let p = GinParams::default();
        assert_eq!(p.base.default_host, Some("127.0.0.1".to_string()));
        assert_eq!(p.base.default_port, Some(8080));
        assert!(p.base.enable_swagger);
        assert!(p.base.enable_cors);
        assert!(p.base.enable_middleware);
        assert!(p.base.enable_logging);
        assert_eq!(p.base.project_name, "");
    }

    #[test]
    fn from_project_name_propagates_name_into_sub_params() {
        let p = GinParams::from_project_name("my-gin".to_string());
        assert_eq!(p.base.project_name, "my-gin");
        assert_eq!(p.base.default_host, Some("127.0.0.1".to_string()));
        assert_eq!(p.base.default_port, Some(8080));
        assert!(p.base.enable_swagger);
        assert!(p.base.enable_cors);
        assert!(p.base.enable_middleware);
        assert!(p.base.enable_logging);
        assert_eq!(
            p.go.base.module_name,
            Some("github.com/example/my-gin".to_string())
        );
    }

    #[test]
    fn with_server_sets_host_and_port() {
        let p = GinParams::from_project_name("svc".to_string())
            .with_server("0.0.0.0".to_string(), 9090);
        assert_eq!(p.base.host, Some("0.0.0.0".to_string()));
        assert_eq!(p.base.port, Some(9090));
    }

    #[test]
    fn with_swagger_and_accessor_round_trip() {
        let on = GinParams::from_project_name("s".to_string()).with_swagger(true);
        assert!(on.enable_swagger());
        let off = GinParams::from_project_name("s".to_string()).with_swagger(false);
        assert!(!off.enable_swagger());
    }

    #[test]
    fn with_cors_toggles_flag() {
        let p = GinParams::from_project_name("s".to_string()).with_cors(false);
        assert!(!p.base.enable_cors);
    }

    #[test]
    fn with_project_and_with_go_replace_sub_params() {
        let project = ProjectParams::from_project_name("proj".to_string());
        let go = GoParams::new("example.com/gomod".to_string());
        let p = GinParams::from_project_name("s".to_string())
            .with_project(project)
            .with_go(go);
        assert_eq!(p.project.base.project_name, "proj");
        assert_eq!(p.go.base.module_name, Some("example.com/gomod".to_string()));
    }

    #[test]
    fn with_database_enables_flag_and_sets_type() {
        let p = GinParams::from_project_name("s".to_string()).with_database("postgres".to_string());
        assert!(p.base.enable_database);
        assert_eq!(p.base.database_type, Some("postgres".to_string()));
    }

    #[test]
    fn with_redis_jwt_precommit_toggle_flags() {
        let p = GinParams::from_project_name("s".to_string())
            .with_redis(true)
            .with_jwt(true)
            .with_precommit(true);
        assert!(p.base.enable_redis);
        assert!(p.base.enable_jwt);
        assert!(p.enable_precommit());
    }

    #[test]
    fn inheritable_params_expose_base_and_render_context() {
        let p = GinParams::from_project_name("ctx-app".to_string());
        assert_eq!(p.base_params().project_name, "ctx-app");
        let ctx = p.to_template_context();
        assert_eq!(ctx["project_name"], serde_json::json!("ctx-app"));
    }
}
