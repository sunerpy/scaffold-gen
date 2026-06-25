//! `scafgen skill` 引擎：把嵌入的 SKILL.md 安装进各 AI agent 的技能目录。
//!
//! 三块职责分离：[`embed`] 提供编译期内联的 SKILL.md + git-blob 哈希 + sidecar
//! 标记；[`engine`] 拥有单技能目录的写/更/卸/查逻辑（仅哈希驱动更新判定）；
//! [`targets`] 把 4 个 agent × Global/Local 解析为技能父目录。CLI 处理器
//! （`commands::skill`）把解析后的子命令映射到这里的函数。

pub mod embed;
pub mod engine;
pub mod targets;

pub use engine::{
    FileAction, SkillStatus, WriteResult, status_for_dir, uninstall_from_dir, write_skill_to_dir,
};
pub use targets::{
    ALL_AGENTS, AgentId, InstallContext, Location, base_config_dir_exists, skill_dir,
};
