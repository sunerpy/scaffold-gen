use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::context::build_base_context;
use super::parameters::Parameters;

/// 基础参数结构 - 包含所有生成器共用的参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseParams {
    // 项目基础信息
    pub project_name: String,
    pub project_version: String,
    pub project_description: Option<String>,
    pub author: Option<String>,
    pub license: String,

    // Git相关
    pub enable_git: bool,
    pub enable_precommit: bool,

    // 服务器配置（适用于Web框架）
    pub host: Option<String>,
    pub port: Option<u16>,

    // 通用功能开关
    pub enable_swagger: bool,
    pub enable_cors: bool,
    pub enable_logging: bool,
    pub enable_recovery: bool,
    pub enable_rate_limit: bool,
    pub enable_jwt: bool,
    pub enable_database: bool,
    pub enable_redis: bool,
    pub enable_grpc: bool,
    pub enable_middleware: bool,

    // 网络配置
    pub default_host: Option<String>,
    pub default_port: Option<u16>,

    // 数据库配置
    pub database_type: Option<String>,

    // 语言特定配置
    pub language_version: Option<String>,
    pub module_name: Option<String>,
    pub enable_modules: bool,
    pub enable_cgo: bool,
    pub build_tags: Vec<String>,
    pub enable_vendor: bool,
}

impl Default for BaseParams {
    fn default() -> Self {
        Self {
            // 项目基础信息
            project_name: String::new(),
            project_version: "0.1.0".to_string(),
            project_description: None,
            author: None,
            license: "MIT".to_string(),

            // Git相关
            enable_git: true,
            enable_precommit: false,

            // 服务器配置
            host: Some("127.0.0.1".to_string()),
            port: Some(8080),

            // 通用功能开关
            enable_swagger: true,
            enable_cors: true,
            enable_logging: true,
            enable_recovery: true,
            enable_rate_limit: false,
            enable_jwt: false,
            enable_database: false,
            enable_redis: false,
            enable_grpc: false,
            enable_middleware: true,

            // 网络配置
            default_host: None,
            default_port: None,

            // 数据库配置
            database_type: None,

            // 语言特定配置
            language_version: Some("1.21".to_string()), // Go默认版本
            module_name: None,
            enable_modules: true,
            enable_cgo: false,
            build_tags: Vec::new(),
            enable_vendor: false,
        }
    }
}

impl Parameters for BaseParams {
    fn validate(&self) -> Result<()> {
        use super::validation;

        validation::validate_project_name(&self.project_name)?;

        if self.license.is_empty() {
            return Err(anyhow::anyhow!("License cannot be empty"));
        }

        if let Some(ref host) = self.host {
            validation::validate_host(host)?;
        }

        if let Some(port) = self.port {
            validation::validate_port(port)?;
        }

        if self.enable_database && self.database_type.is_none() {
            return Err(anyhow::anyhow!(
                "Database type must be specified when database is enabled"
            ));
        }

        Ok(())
    }

    fn to_template_context(&self) -> HashMap<String, Value> {
        build_base_context(self)
    }
}

impl BaseParams {
    /// 创建新的基础参数
    pub fn new(project_name: String) -> Self {
        Self {
            project_name: project_name.clone(),
            module_name: Some(project_name),
            ..Default::default()
        }
    }

    /// 设置项目描述
    pub fn with_description(mut self, description: String) -> Self {
        self.project_description = Some(description);
        self
    }

    /// 设置作者
    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }

    /// 设置许可证
    pub fn with_license(mut self, license: String) -> Self {
        self.license = license;
        self
    }

    /// 设置数据库配置
    /// allow(dead_code): no production caller yet, but locked by the Phase 1
    /// safety-net test `to_template_context_includes_author_and_database_when_present`.
    #[allow(dead_code)]
    pub fn with_database(mut self, db_type: String) -> Self {
        self.enable_database = true;
        self.database_type = Some(db_type);
        self
    }
}

/// 参数继承trait - 用于扩展基础参数
pub trait InheritableParams: Parameters {
    /// 获取基础参数的引用
    fn base_params(&self) -> &BaseParams;

    /// 获取扩展的模板上下文（子类特有的参数）
    fn extended_template_context(&self) -> HashMap<String, Value> {
        HashMap::new()
    }
}

/// 为实现了InheritableParams的类型提供默认的Parameters实现
impl<T: InheritableParams> Parameters for T {
    fn validate(&self) -> Result<()> {
        // 首先验证基础参数
        self.base_params().validate()?;

        // 子类可以重写此方法添加额外验证
        Ok(())
    }

    fn to_template_context(&self) -> HashMap<String, Value> {
        let mut context = self.base_params().to_template_context();

        // 合并扩展的模板上下文
        let extended_context = self.extended_template_context();
        context.extend(extended_context);

        context
    }
}
