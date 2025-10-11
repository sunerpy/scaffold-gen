use anyhow::Result;
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::generators::core::{Parameters, validation};

/// 项目级别参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectParams {
    /// 项目名称
    pub name: String,
    /// 项目描述
    pub description: Option<String>,
    /// 作者信息
    pub author: Option<String>,
    /// 许可证类型
    pub license: String,
    /// 是否启用Git
    pub enable_git: bool,
    /// 项目版本
    pub version: String,
}

impl Default for ProjectParams {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            author: None,
            license: "MIT".to_string(),
            enable_git: true,
            version: "0.1.0".to_string(),
        }
    }
}

impl Parameters for ProjectParams {
    fn validate(&self) -> Result<()> {
        validation::validate_project_name(&self.name)?;

        if self.license.is_empty() {
            return Err(anyhow::anyhow!("License cannot be empty"));
        }

        Ok(())
    }

    fn to_template_context(&self) -> HashMap<String, Value> {
        let mut context = HashMap::new();

        context.insert("project_name".to_string(), json!(self.name));
        context.insert("project_version".to_string(), json!(self.version));
        context.insert("license".to_string(), json!(self.license));
        context.insert("enable_git".to_string(), json!(self.enable_git));

        // 添加项目名称的不同格式
        context.insert(
            "project_name_pascal".to_string(),
            json!(crate::constants::string_utils::to_pascal_case(&self.name)),
        );
        context.insert(
            "project_name_snake".to_string(),
            json!(crate::constants::string_utils::to_snake_case(&self.name)),
        );

        if let Some(ref description) = self.description {
            context.insert("project_description".to_string(), json!(description));
        }

        if let Some(ref author) = self.author {
            context.insert("author".to_string(), json!(author));
        }

        // 添加当前年份用于LICENSE文件
        let current_year = chrono::Utc::now().year();
        context.insert("year".to_string(), json!(current_year));

        context
    }

    fn override_from_env(&mut self) -> Result<()> {
        if let Ok(author) = std::env::var("GIT_AUTHOR_NAME") {
            self.author = Some(author);
        }

        if let Ok(license) = std::env::var("DEFAULT_LICENSE") {
            self.license = license;
        }

        Ok(())
    }
}

impl ProjectParams {
    /// 创建新的项目参数
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    /// 设置项目描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
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

    /// 设置是否启用Git
    pub fn with_git(mut self, enable_git: bool) -> Self {
        self.enable_git = enable_git;
        self
    }

    /// 设置版本
    #[allow(dead_code)]
    pub fn with_version(mut self, version: String) -> Self {
        self.version = version;
        self
    }
}
