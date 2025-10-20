use serde::{Deserialize, Serialize};

use crate::generators::core::{BaseParams, InheritableParams};

/// 项目级别参数 - 现在继承自BaseParams
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectParams {
    /// 基础参数
    pub base: BaseParams,
}

impl InheritableParams for ProjectParams {
    fn base_params(&self) -> &BaseParams {
        &self.base
    }

    fn base_params_mut(&mut self) -> &mut BaseParams {
        &mut self.base
    }

    fn from_base(base: BaseParams) -> Self {
        Self { base }
    }

    // ProjectParams没有额外的参数，所以不需要重写extended_template_context
}

impl ProjectParams {
    /// 创建新的项目参数
    pub fn new(name: String) -> Self {
        let mut base = BaseParams::new(name.clone());
        // 设置项目特定的默认值
        base.enable_git = true;
        base.enable_precommit = false;

        Self { base }
    }

    /// 从项目名称创建
    pub fn from_project_name(project_name: String) -> Self {
        Self::new(project_name)
    }

    /// 设置项目描述
    pub fn with_description(mut self, description: String) -> Self {
        self.base = self.base.with_description(description);
        self
    }

    /// 设置作者
    pub fn with_author(mut self, author: String) -> Self {
        self.base = self.base.with_author(author);
        self
    }

    /// 设置许可证
    pub fn with_license(mut self, license: String) -> Self {
        self.base = self.base.with_license(license);
        self
    }

    /// 设置是否启用Git
    pub fn with_git(mut self, enable_git: bool) -> Self {
        self.base.enable_git = enable_git;
        self
    }

    /// 设置是否启用pre-commit hooks
    pub fn with_precommit(mut self, enable_precommit: bool) -> Self {
        self.base.enable_precommit = enable_precommit;
        self
    }

    /// 设置版本
    #[allow(dead_code)]
    pub fn with_version(mut self, version: String) -> Self {
        self.base.project_version = version;
        self
    }

    // 为了向后兼容，提供访问器方法
    pub fn name(&self) -> &str {
        &self.base.project_name
    }

    pub fn description(&self) -> &Option<String> {
        &self.base.project_description
    }

    pub fn author(&self) -> &Option<String> {
        &self.base.author
    }

    pub fn license(&self) -> &str {
        &self.base.license
    }

    pub fn enable_git(&self) -> bool {
        self.base.enable_git
    }

    pub fn enable_precommit(&self) -> bool {
        self.base.enable_precommit
    }

    pub fn version(&self) -> &str {
        &self.base.project_version
    }
}
