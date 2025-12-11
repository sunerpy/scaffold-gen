use serde::{Deserialize, Serialize};

use crate::generators::core::{BaseParams, InheritableParams};
use crate::generators::language::go::GoParams;
use crate::generators::project::ProjectParams;

/// Go-Zero框架参数 - 现在继承自BaseParams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoZeroParams {
    /// 基础参数
    pub base: BaseParams,
    /// 项目级别参数
    pub project: ProjectParams,
    /// Go语言参数
    pub go: GoParams,
    /// Go-Zero特有的服务开关
    pub enable_api: bool,
    pub enable_rpc: bool,
    pub enable_admin: bool,
}

impl Default for GoZeroParams {
    fn default() -> Self {
        let base = BaseParams {
            default_host: Some("127.0.0.1".to_string()),
            default_port: Some(8888),
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
            enable_api: true,
            enable_rpc: false,
            enable_admin: false,
        }
    }
}

impl InheritableParams for GoZeroParams {
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
            enable_api: true,
            enable_rpc: false,
            enable_admin: false,
        }
    }

    // Go-Zero参数有额外的project和go参数
}

impl GoZeroParams {
    /// 创建新的Go-Zero参数
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// 从项目名称创建
    #[allow(dead_code)]
    pub fn from_project_name(project_name: String) -> Self {
        let mut base = BaseParams::new(project_name.clone());
        // 设置Go-Zero特定的默认值
        base.default_host = Some("127.0.0.1".to_string());
        base.default_port = Some(8888);
        base.enable_swagger = true;
        base.enable_cors = true;
        base.enable_middleware = true;
        base.enable_logging = true;
        base.enable_grpc = true;

        Self {
            base,
            project: ProjectParams::from_project_name(project_name.clone()),
            go: GoParams::from_project_name(project_name),
            enable_api: true,
            enable_rpc: false,
            enable_admin: false,
        }
    }

    /// 设置主机地址
    #[allow(dead_code)]
    pub fn with_host(mut self, host: String) -> Self {
        self.base.default_host = Some(host);
        self
    }

    /// 设置端口
    #[allow(dead_code)]
    pub fn with_port(mut self, port: u16) -> Self {
        self.base.default_port = Some(port);
        self
    }

    /// 设置是否启用Swagger
    #[allow(dead_code)]
    pub fn with_swagger(mut self, enable_swagger: bool) -> Self {
        self.base.enable_swagger = enable_swagger;
        self
    }

    /// 设置是否启用CORS
    #[allow(dead_code)]
    pub fn with_cors(mut self, enable_cors: bool) -> Self {
        self.base.enable_cors = enable_cors;
        self
    }

    /// 设置是否启用中间件
    #[allow(dead_code)]
    pub fn with_middleware(mut self, enable_middleware: bool) -> Self {
        self.base.enable_middleware = enable_middleware;
        self
    }

    /// 设置是否启用日志
    #[allow(dead_code)]
    pub fn with_logging(mut self, enable_logging: bool) -> Self {
        self.base.enable_logging = enable_logging;
        self
    }

    /// 设置是否启用gRPC
    #[allow(dead_code)]
    pub fn with_grpc(mut self, enable_grpc: bool) -> Self {
        self.base.enable_grpc = enable_grpc;
        self
    }

    /// 设置是否启用Admin服务
    #[allow(dead_code)]
    pub fn with_admin(mut self, enable_admin: bool) -> Self {
        self.enable_admin = enable_admin;
        self
    }

    #[allow(dead_code)]
    pub fn with_api(mut self, enable_api: bool) -> Self {
        self.enable_api = enable_api;
        self
    }

    #[allow(dead_code)]
    pub fn with_rpc(mut self, enable_rpc: bool) -> Self {
        self.enable_rpc = enable_rpc;
        self
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

    // 为了向后兼容，提供访问器方法
    #[allow(dead_code)]
    pub fn host(&self) -> Option<&String> {
        self.base.default_host.as_ref()
    }

    #[allow(dead_code)]
    pub fn port(&self) -> Option<u16> {
        self.base.port
    }

    #[allow(dead_code)]
    pub fn enable_swagger(&self) -> bool {
        self.base.enable_swagger
    }

    #[allow(dead_code)]
    pub fn enable_cors(&self) -> bool {
        self.base.enable_cors
    }

    #[allow(dead_code)]
    pub fn enable_middleware(&self) -> bool {
        self.base.enable_middleware
    }

    #[allow(dead_code)]
    pub fn enable_logging(&self) -> bool {
        self.base.enable_logging
    }

    #[allow(dead_code)]
    pub fn enable_grpc(&self) -> bool {
        self.base.enable_grpc
    }

    pub fn enable_admin(&self) -> bool {
        self.enable_admin
    }

    pub fn enable_api(&self) -> bool {
        self.enable_api
    }

    pub fn enable_rpc(&self) -> bool {
        self.enable_rpc
    }
}
