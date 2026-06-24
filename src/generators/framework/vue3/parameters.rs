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
}

impl Vue3Params {
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
}
