use serde::{Deserialize, Serialize};

use crate::generators::core::{BaseParams, InheritableParams};
use crate::generators::project::ProjectParams;

/// Vue3框架参数 - 继承自BaseParams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vue3Params {
    /// 基础参数
    pub base: BaseParams,
    /// 项目级别参数
    pub project: ProjectParams,
    /// Node.js 版本
    pub node_version: String,
    /// 是否启用 TypeScript (强制启用)
    pub enable_typescript: bool,
    /// 是否启用 Tailwind CSS
    pub enable_tailwind: bool,
    /// 是否启用 Vue Router
    pub enable_router: bool,
    /// 是否启用 Pinia 状态管理
    pub enable_pinia: bool,
    /// 是否启用 ESLint
    pub enable_eslint: bool,
    /// 是否启用 Prettier
    pub enable_prettier: bool,
    /// 包管理器 (pnpm)
    pub package_manager: String,
}

impl Default for Vue3Params {
    fn default() -> Self {
        let base = BaseParams {
            default_host: Some("localhost".to_string()),
            default_port: Some(5173),
            ..Default::default()
        };

        Self {
            base,
            project: ProjectParams::default(),
            node_version: "20".to_string(),
            enable_typescript: true, // 强制启用 TypeScript
            enable_tailwind: true,
            enable_router: true,
            enable_pinia: true,
            enable_eslint: true,
            enable_prettier: true,
            package_manager: "pnpm".to_string(),
        }
    }
}

impl InheritableParams for Vue3Params {
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
            node_version: "20".to_string(),
            enable_typescript: true,
            enable_tailwind: true,
            enable_router: true,
            enable_pinia: true,
            enable_eslint: true,
            enable_prettier: true,
            package_manager: "pnpm".to_string(),
        }
    }
}

impl Vue3Params {
    /// 创建新的Vue3参数
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// 从项目名称创建
    pub fn from_project_name(project_name: String) -> Self {
        let mut base = BaseParams::new(project_name.clone());
        base.default_host = Some("localhost".to_string());
        base.default_port = Some(5173);

        Self {
            base,
            project: ProjectParams::from_project_name(project_name),
            node_version: "20".to_string(),
            enable_typescript: true,
            enable_tailwind: true,
            enable_router: true,
            enable_pinia: true,
            enable_eslint: true,
            enable_prettier: true,
            package_manager: "pnpm".to_string(),
        }
    }

    /// 设置项目参数
    pub fn with_project(mut self, project: ProjectParams) -> Self {
        self.project = project;
        self
    }

    /// 设置是否启用pre-commit
    pub fn with_precommit(mut self, enable_precommit: bool) -> Self {
        self.base.enable_precommit = enable_precommit;
        self
    }

    /// 获取是否启用pre-commit
    pub fn enable_precommit(&self) -> bool {
        self.base.enable_precommit
    }

    /// 设置是否启用 Tailwind CSS
    #[allow(dead_code)]
    pub fn with_tailwind(mut self, enable: bool) -> Self {
        self.enable_tailwind = enable;
        self
    }

    /// 设置是否启用 Vue Router
    #[allow(dead_code)]
    pub fn with_router(mut self, enable: bool) -> Self {
        self.enable_router = enable;
        self
    }

    /// 设置是否启用 Pinia
    #[allow(dead_code)]
    pub fn with_pinia(mut self, enable: bool) -> Self {
        self.enable_pinia = enable;
        self
    }

    /// 设置 Node.js 版本
    #[allow(dead_code)]
    pub fn with_node_version(mut self, version: String) -> Self {
        self.node_version = version;
        self
    }
}
