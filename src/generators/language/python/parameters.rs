// TODO: Implement Python parameters
// This is a placeholder for future Python project generation support

use serde::{Deserialize, Serialize};

use crate::generators::core::{BaseParams, InheritableParams};

/// Python语言级别参数 - 待实现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonParams {
    /// 基础参数
    pub base: BaseParams,
}

impl Default for PythonParams {
    fn default() -> Self {
        let base = BaseParams {
            language_version: Some("3.11".to_string()),
            ..Default::default()
        };

        Self { base }
    }
}

impl InheritableParams for PythonParams {
    fn base_params(&self) -> &BaseParams {
        &self.base
    }

    fn base_params_mut(&mut self) -> &mut BaseParams {
        &mut self.base
    }

    fn from_base(base: BaseParams) -> Self {
        Self { base }
    }
}
