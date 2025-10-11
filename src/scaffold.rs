use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::template_engine::TemplateEngine;

/// 参数作用域，用于管理模板参数
#[derive(Debug, Clone)]
pub struct ParameterScope {
    params: HashMap<String, Value>,
}

impl ParameterScope {
    /// 创建新的参数作用域
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    /// 添加参数
    pub fn add<T: Into<Value>>(&mut self, key: &str, value: T) -> &mut Self {
        self.params.insert(key.to_string(), value.into());
        self
    }

    /// 批量添加参数
    #[allow(dead_code)]
    pub fn add_all(&mut self, params: HashMap<String, Value>) -> &mut Self {
        self.params.extend(params);
        self
    }

    /// 获取参数
    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.params.get(key)
    }

    /// 获取所有参数
    pub fn get_all(&self) -> &HashMap<String, Value> {
        &self.params
    }

    /// 合并另一个参数作用域
    #[allow(dead_code)]
    pub fn merge(&mut self, other: ParameterScope) -> &mut Self {
        self.params.extend(other.params);
        self
    }
}

impl Default for ParameterScope {
    fn default() -> Self {
        Self::new()
    }
}

/// 脚手架生成器核心类
pub struct Scaffold {
    template_path: PathBuf,
    output_path: Option<PathBuf>,
    params: ParameterScope,
    template_engine: TemplateEngine,
    post_processors: Vec<PostProcessor>,
}

impl Scaffold {
    /// 创建新的脚手架生成器
    pub fn new<P: AsRef<Path>>(template_path: P) -> Result<Self> {
        let template_path = template_path.as_ref().to_path_buf();

        // 获取模板根目录
        let templates_root = crate::template_engine::get_templates_dir()?;

        // 构建完整的模板路径
        let full_template_path = if template_path.is_absolute() {
            template_path.clone()
        } else {
            templates_root.join(&template_path)
        };

        // 检查模板目录是否存在
        if !full_template_path.exists() {
            return Err(anyhow::anyhow!(
                "Template directory not found: {}\nExpected path: {}",
                template_path.display(),
                full_template_path.display()
            ));
        }

        let template_engine = TemplateEngine::new(templates_root)?;

        Ok(Self {
            template_path: full_template_path,
            output_path: None,
            params: ParameterScope::new(),
            template_engine,
            post_processors: Vec::new(),
        })
    }

    /// 设置输出路径
    pub fn output_to<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.output_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// 设置参数
    pub fn with_params(mut self, params: ParameterScope) -> Self {
        self.params = params;
        self
    }

    /// 添加单个参数
    #[allow(dead_code)]
    pub fn with_param<T: Into<Value>>(mut self, key: &str, value: T) -> Self {
        self.params.add(key, value);
        self
    }

    /// 添加后置处理器
    #[allow(dead_code)]
    pub fn with_post_processor(mut self, processor: PostProcessor) -> Self {
        self.post_processors.push(processor);
        self
    }

    /// 处理模板并生成文件
    pub fn process(mut self) -> Result<ProcessedScaffold> {
        let output_path = self
            .output_path
            .take()
            .unwrap_or_else(|| std::env::current_dir().unwrap());

        // 确保输出目录存在
        std::fs::create_dir_all(&output_path).with_context(|| {
            format!(
                "Failed to create output directory: {}",
                output_path.display()
            )
        })?;

        // 处理模板文件
        self.process_templates(&output_path)?;

        Ok(ProcessedScaffold {
            output_path,
            post_processors: self.post_processors,
        })
    }

    /// 递归处理模板文件
    fn process_templates(&mut self, output_path: &Path) -> Result<()> {
        self.process_directory(&self.template_path.clone(), output_path, "")?;
        Ok(())
    }

    /// 递归处理目录
    fn process_directory(
        &mut self,
        template_dir: &Path,
        output_dir: &Path,
        relative_path: &str,
    ) -> Result<()> {
        for entry in std::fs::read_dir(template_dir).with_context(|| {
            format!(
                "Failed to read template directory: {}",
                template_dir.display()
            )
        })? {
            let entry = entry?;
            let path = entry.path();
            let file_name_os = entry.file_name();
            let file_name = file_name_os.to_string_lossy();

            // 只跳过构建系统相关的特殊文件，允许所有模板文件被处理
            if file_name == "Cargo.toml" || file_name == "Cargo.lock" {
                continue;
            }

            let current_relative = if relative_path.is_empty() {
                file_name.to_string()
            } else {
                format!("{relative_path}/{file_name}")
            };

            if path.is_dir() {
                let new_output_dir = output_dir.join(&*file_name);
                std::fs::create_dir_all(&new_output_dir).with_context(|| {
                    format!("Failed to create directory: {}", new_output_dir.display())
                })?;
                self.process_directory(&path, &new_output_dir, &current_relative)?;
            } else {
                self.process_file(&path, output_dir, &file_name)?;
            }
        }
        Ok(())
    }

    /// 处理单个文件
    fn process_file(
        &mut self,
        template_file: &Path,
        output_dir: &Path,
        file_name: &str,
    ) -> Result<()> {
        // 检查是否应该跳过此文件
        if self.should_skip_file(file_name) {
            println!("⏭️  Skipped: {file_name} (disabled by configuration)");
            return Ok(());
        }

        let output_file_name = file_name.strip_suffix(".tmpl").unwrap_or(file_name);

        let output_file = output_dir.join(output_file_name);

        if file_name.ends_with(".tmpl") {
            // 处理模板文件 - 直接使用模板文件的绝对路径
            let content = self
                .template_engine
                .render_template(template_file, self.params.get_all())
                .with_context(|| {
                    format!("Failed to render template: {}", template_file.display())
                })?;

            std::fs::write(&output_file, content)
                .with_context(|| format!("Failed to write file: {}", output_file.display()))?;
        } else {
            // 直接复制非模板文件
            std::fs::copy(template_file, &output_file).with_context(|| {
                format!(
                    "Failed to copy file: {} -> {}",
                    template_file.display(),
                    output_file.display()
                )
            })?;
        }

        println!("✅ Generated: {}", output_file.display());
        Ok(())
    }

    /// 检查是否应该跳过文件
    fn should_skip_file(&self, file_name: &str) -> bool {
        // 检查 pre-commit 配置文件
        if file_name == ".pre-commit-config.yaml.tmpl" || file_name == ".pre-commit-config.yaml" {
            if let Some(enable_precommit) = self.params.get("enable_precommit") {
                if let Some(enabled) = enable_precommit.as_bool() {
                    return !enabled;
                }
            }
        }

        // 可以在这里添加更多的条件检查
        // 例如：数据库相关文件等

        false
    }
}

/// 已处理的脚手架，可以执行后置处理器
pub struct ProcessedScaffold {
    output_path: PathBuf,
    post_processors: Vec<PostProcessor>,
}

impl ProcessedScaffold {
    /// 运行后置处理器
    pub fn run_post_processors(self) -> Result<CompletedScaffold> {
        for processor in &self.post_processors {
            processor.execute(&self.output_path)?;
        }

        Ok(CompletedScaffold {
            output_path: self.output_path,
        })
    }

    /// 获取输出路径
    #[allow(dead_code)]
    pub fn output_path(&self) -> &Path {
        &self.output_path
    }
}

/// 完成的脚手架
#[allow(dead_code)]
pub struct CompletedScaffold {
    output_path: PathBuf,
}

impl CompletedScaffold {
    /// 获取输出路径
    #[allow(dead_code)]
    pub fn output_path(&self) -> &Path {
        &self.output_path
    }
}

/// 后置处理器
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PostProcessor {
    /// 执行自定义命令
    Command {
        command: String,
        args: Vec<String>,
        description: String,
    },
}

impl PostProcessor {
    /// 创建命令处理器
    #[allow(dead_code)]
    pub fn command<S: Into<String>>(command: S, args: Vec<S>, description: S) -> Self {
        Self::Command {
            command: command.into(),
            args: args.into_iter().map(|s| s.into()).collect(),
            description: description.into(),
        }
    }

    /// 执行后置处理器
    pub fn execute(&self, output_path: &Path) -> Result<()> {
        match self {
            PostProcessor::Command {
                command,
                args,
                description,
            } => {
                println!("🔧 {description}");
                let output = Command::new(command)
                    .args(args)
                    .current_dir(output_path)
                    .output()
                    .with_context(|| format!("Failed to execute command: {command} {args:?}"))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(anyhow::anyhow!(
                        "Command failed: {description}\nError: {stderr}"
                    ));
                }
                println!("✅ {description}");
            }
        }
        Ok(())
    }
}
