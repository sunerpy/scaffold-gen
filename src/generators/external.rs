//! 外部脚手架生成路径（Tauri）—— 会 shell 到 create-tauri-app / pnpm。
//!
//! 从 `orchestrator.rs` 拆出：这些方法都属于 `GeneratorOrchestrator`，
//! 通过 `generate_external` 按框架分派进来。`print_external_completion`
//! 是完成提示尾段（Tauri 使用）。

use anyhow::{Context, Result};
use std::path::Path;

use crate::generators::core::Generator;
use crate::generators::framework::tauri::{TauriGenerator, TauriParams};
use crate::generators::orchestrator::{GenerationRequest, GeneratorOrchestrator};
use crate::generators::project::ProjectParams;

impl GeneratorOrchestrator {
    /// 生成完整的Tauri项目
    pub(super) async fn generate_tauri_project(
        &mut self,
        request: &GenerationRequest<'_>,
    ) -> Result<()> {
        let project_name = &request.project_name;
        let output_path = request.output_path;
        let enable_precommit = request.enable_precommit;

        tracing::info!("Starting Tauri project generation: {project_name}");

        // 1. 环境预检查
        tracing::debug!("🔍 Checking environment prerequisites...");

        // 检查 pnpm
        if !TauriGenerator::check_pnpm()? {
            return Err(anyhow::anyhow!(
                "pnpm is not installed. Please install pnpm first:\n  npm install -g pnpm\n  or visit: https://pnpm.io/installation"
            ));
        }
        tracing::debug!("  ✅ pnpm: Available");

        // 检查 create-tauri-app
        if !TauriGenerator::check_create_tauri_app()? {
            tracing::debug!("  ⚠️ create-tauri-app not found, installing...");
            TauriGenerator::install_create_tauri_app()?;
        }
        tracing::debug!("  ✅ create-tauri-app: Available");

        // 2. 删除已存在的目录（如果存在）
        if output_path.exists() {
            std::fs::remove_dir_all(output_path).context("Failed to remove existing directory")?;
        }

        // 3. 使用 create-tauri-app 创建项目
        TauriGenerator::create_tauri_project(project_name, output_path)?;

        // 4. 安装前端依赖
        TauriGenerator::install_dependencies(output_path)?;

        // 5. 创建项目参数
        let project_params = ProjectParams::new(project_name.clone())
            .with_license(request.license.clone())
            .with_git(true)
            .with_precommit(enable_precommit)
            .with_description(request.spec.description(project_name));

        // 6. 创建 Tauri 参数
        let tauri_params = TauriParams::from_project_name(project_name.clone())
            .with_project(project_params.clone())
            .with_precommit(enable_precommit)
            .with_proto_gen(request.enable_proto_gen)
            .with_error_gen(request.enable_error_gen);

        // 7. 覆盖模板文件 - 添加骨架屏、Tailwind CSS 等功能
        tracing::debug!("📝 Applying enhanced templates...");
        TauriGenerator::new()?
            .generate(tauri_params, output_path)
            .context("Failed to apply Tauri templates")?;

        // 8. 重新安装依赖（因为 package.json 可能已更新）
        tracing::debug!("📦 Reinstalling dependencies with updated package.json...");
        TauriGenerator::install_dependencies(output_path)?;

        // 9. 项目级别生成 - 生成 LICENSE 等
        self.project_generator
            .generate(project_params, output_path)
            .context("Failed to generate project files")?;

        print_external_completion("Tauri", output_path, project_name, request.spec.next_steps);

        Ok(())
    }
}

/// 外部脚手架框架的完成提示 —— Tauri 生成结束后打印的 4 行尾段。
fn print_external_completion(
    label: &str,
    output_path: &Path,
    project_name: &str,
    next_steps: &[&str],
) {
    tracing::info!("✅ {label} project generation completed successfully!");
    tracing::info!("📁 Project created at: {}", output_path.display());
    tracing::info!("\n📋 Next steps:");
    tracing::info!("  cd {project_name}");
    for line in next_steps {
        tracing::info!("{line}");
    }
}
