//! `scafgen skill <install|update|uninstall|status>` 命令处理器。
//!
//! 把解析后的子命令映射到 [`crate::skill`] 引擎调用：解析 `--target` 子集、决定
//! Global/Local 位置、对每个 agent×位置解析技能父目录，再调用写/更/卸/查。
//! 所有人类可读输出走 `tracing`（与项目 `-q/-v` 约定一致）；安装/卸载非交互、
//! 不阻塞（agent 会话里绝不能挂起）。

use anyhow::{Result, anyhow};

use crate::skill::{
    self, AgentId, FileAction, InstallContext, Location, SkillStatus, WriteResult,
    base_config_dir_exists,
};

/// 解析出的“做什么”。从 main.rs 的 `SkillAction` 扁平化映射而来，避免处理器与
/// clap 派生强耦合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Install,
    Update,
    Uninstall,
    Status,
}

/// 一次 skill 子命令的已归一化参数。
pub struct SkillRequest {
    pub action: Action,
    /// `--target` 选中的 agent；空 = 全部（按检测/显式规则）。
    pub targets: Vec<String>,
    /// 是否显式 `--local`（否则默认 Global）。
    pub local: bool,
    /// `--force`（仅 update：覆盖本地修改）。
    pub force: bool,
}

/// 入口：归一化请求 → 逐 agent×位置执行 → tracing 报告。
pub fn execute(request: SkillRequest) -> Result<()> {
    let ctx = InstallContext::from_env()?;
    let location = if request.local {
        Location::Local
    } else {
        Location::Global
    };

    let explicit = !request.targets.is_empty();
    let agents = resolve_targets(&request.targets)?;

    // 未显式指定 target 时，仅对基准配置目录已存在的 agent 自动作用（避免乱建目录）；
    // 显式指定则无条件作用于这些 agent。
    let selected: Vec<AgentId> = agents
        .into_iter()
        .filter(|&a| explicit || base_config_dir_exists(a, &ctx, location))
        .collect();

    if selected.is_empty() {
        tracing::warn!(
            "No matching agent config directories found for --location={}. \
             Pass --target <opencode|claude|cursor|kiro> to write anyway.",
            location.as_str()
        );
        return Ok(());
    }

    for agent in &selected {
        let dir = skill::skill_dir(*agent, &ctx, location);
        match request.action {
            Action::Install => {
                let result = skill::write_skill_to_dir(&dir, false);
                report_write(*agent, location, "install", &result);
            }
            Action::Update => {
                let result = skill::write_skill_to_dir(&dir, request.force);
                report_write(*agent, location, "update", &result);
            }
            Action::Uninstall => {
                let result = skill::uninstall_from_dir(&dir);
                report_write(*agent, location, "uninstall", &result);
            }
            Action::Status => {
                let status = skill::status_for_dir(&dir);
                report_status(*agent, location, status);
            }
        }
    }

    if matches!(request.action, Action::Install | Action::Update) {
        tracing::info!("Done. Restart your agent(s) to start using the scaffold-gen skill.");
    }

    Ok(())
}

/// 把 `--target` 字符串子集解析为去重后的 agent 列表（保持 `ALL_AGENTS` 顺序）；
/// 空列表 = 全部四个。未知名报清晰错误。
fn resolve_targets(targets: &[String]) -> Result<Vec<AgentId>> {
    if targets.is_empty() {
        return Ok(skill::ALL_AGENTS.to_vec());
    }
    let mut selected = Vec::new();
    for raw in targets {
        let agent = AgentId::parse(raw).ok_or_else(|| {
            let valid = skill::ALL_AGENTS
                .iter()
                .map(|a| a.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            anyhow!("unknown --target '{raw}'. valid targets: {valid}")
        })?;
        if !selected.contains(&agent) {
            selected.push(agent);
        }
    }
    // 按稳定顺序输出。
    Ok(skill::ALL_AGENTS
        .iter()
        .copied()
        .filter(|a| selected.contains(a))
        .collect())
}

/// 逐 agent 报告一次写/更/卸结果。
fn report_write(agent: AgentId, loc: Location, verb: &str, result: &WriteResult) {
    for (path, action) in &result.files {
        match action {
            FileAction::Created | FileAction::Updated | FileAction::Removed => {
                tracing::info!(
                    "[{} {}] {} {}: {}",
                    agent.display_name(),
                    loc.as_str(),
                    verb,
                    action.verb(),
                    path.display()
                );
            }
            FileAction::Unchanged | FileAction::Skipped | FileAction::NotFound => {
                tracing::info!(
                    "[{} {}] {} {}",
                    agent.display_name(),
                    loc.as_str(),
                    verb,
                    action.verb()
                );
            }
        }
    }
    for note in &result.notes {
        tracing::warn!("[{} {}] {note}", agent.display_name(), loc.as_str());
    }
}

/// 逐 agent 报告一次状态查询。
fn report_status(agent: AgentId, loc: Location, status: SkillStatus) {
    tracing::info!(
        "[{} {}] {}",
        agent.display_name(),
        loc.as_str(),
        status.label()
    );
}

#[cfg(test)]
#[path = "skill_tests.rs"]
mod tests;
