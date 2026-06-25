//! Agent 目标与按 agent / 位置解析技能父目录。
//!
//! 支持 4 个 agent：opencode / claude / cursor / kiro。每个 agent 在 Global /
//! Local 两个位置下解析出一个“技能父目录”，引擎再在其下写
//! `scaffold-gen/SKILL.md` 与 sidecar。所有路径都从注入的 [`InstallContext`]
//! （home / cwd / XDG）派生，不在逻辑深处直接读 `$HOME`，以便测试注入临时目录。

use std::path::PathBuf;

/// 安装位置：全局（用户级）或本地（项目级 cwd）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    Global,
    Local,
}

impl Location {
    pub fn as_str(self) -> &'static str {
        match self {
            Location::Global => "global",
            Location::Local => "local",
        }
    }
}

/// 受支持的 agent 稳定标识（用于 `--target` 与报告）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentId {
    Opencode,
    Claude,
    Cursor,
    Kiro,
}

/// 全部 agent，顺序稳定（默认全选 / 报告均按此顺序）。
pub const ALL_AGENTS: [AgentId; 4] = [
    AgentId::Opencode,
    AgentId::Claude,
    AgentId::Cursor,
    AgentId::Kiro,
];

impl AgentId {
    /// `--target` 取值与报告标识。
    pub fn as_str(self) -> &'static str {
        match self {
            AgentId::Opencode => "opencode",
            AgentId::Claude => "claude",
            AgentId::Cursor => "cursor",
            AgentId::Kiro => "kiro",
        }
    }

    /// 人类可读显示名。
    pub fn display_name(self) -> &'static str {
        match self {
            AgentId::Opencode => "opencode",
            AgentId::Claude => "Claude Code",
            AgentId::Cursor => "Cursor",
            AgentId::Kiro => "Kiro",
        }
    }

    /// 解析 `--target` 字符串为 agent（未知名返回 `None`，由调用方报清晰错误）。
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "opencode" => Some(AgentId::Opencode),
            "claude" => Some(AgentId::Claude),
            "cursor" => Some(AgentId::Cursor),
            "kiro" => Some(AgentId::Kiro),
            _ => None,
        }
    }
}

/// 路径解析所依赖的文件系统根。
///
/// 把 home / cwd / XDG 线程化进来（而非在逻辑深处读环境），让 per-agent 路径逻辑
/// 可针对临时目录测试，无需 `chdir` / `setenv` 竞争。
#[derive(Debug, Clone)]
pub struct InstallContext {
    pub home: PathBuf,
    pub cwd: PathBuf,
    /// `$XDG_CONFIG_HOME`，仅用于 opencode 的全局配置目录。
    pub xdg_config_home: Option<PathBuf>,
}

impl InstallContext {
    /// 从真实环境构造：`$HOME` / 当前目录 / `$XDG_CONFIG_HOME`。
    pub fn from_env() -> std::io::Result<Self> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .or_else(dirs_home)
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "could not resolve home dir")
            })?;
        let cwd = std::env::current_dir()?;
        let xdg_config_home = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty());
        Ok(Self {
            home,
            cwd,
            xdg_config_home,
        })
    }
}

/// `$HOME` 缺失时的兜底（Windows 等）。
fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE").map(PathBuf::from)
}

/// opencode 的全局配置目录：`$XDG_CONFIG_HOME` 优先，否则 `~/.config`，再进 `opencode`。
fn opencode_global_config_dir(ctx: &InstallContext) -> PathBuf {
    let xdg = ctx
        .xdg_config_home
        .clone()
        .unwrap_or_else(|| ctx.home.join(".config"));
    xdg.join("opencode")
}

/// 解析某 agent 在某位置下的“技能父目录”（引擎在其下追加 `scaffold-gen/SKILL.md`）。
///
/// 注意 opencode 用单数 `skill`，其余三家用复数 `skills`。
pub fn skill_dir(agent: AgentId, ctx: &InstallContext, loc: Location) -> PathBuf {
    match agent {
        AgentId::Opencode => match loc {
            // opencode 用 SINGULAR `skill`。
            Location::Global => opencode_global_config_dir(ctx).join("skill"),
            Location::Local => ctx.cwd.join(".opencode").join("skill"),
        },
        AgentId::Claude => match loc {
            Location::Global => ctx.home.join(".claude").join("skills"),
            Location::Local => ctx.cwd.join(".claude").join("skills"),
        },
        AgentId::Cursor => match loc {
            Location::Global => ctx.home.join(".cursor").join("skills"),
            Location::Local => ctx.cwd.join(".cursor").join("skills"),
        },
        AgentId::Kiro => match loc {
            Location::Global => ctx.home.join(".kiro").join("skills"),
            Location::Local => ctx.cwd.join(".kiro").join("skills"),
        },
    }
}

/// agent 的基准配置目录是否存在（用于“自动检测，显式覆盖”的默认安装）。
///
/// 未给 `--target` 时，只对基准配置目录已存在的 agent 自动安装，避免给用户没装的
/// agent 乱建目录；显式 `--target` 时无条件写入。
pub fn base_config_dir_exists(agent: AgentId, ctx: &InstallContext, loc: Location) -> bool {
    let base = match agent {
        AgentId::Opencode => match loc {
            Location::Global => opencode_global_config_dir(ctx),
            Location::Local => ctx.cwd.join(".opencode"),
        },
        AgentId::Claude => match loc {
            Location::Global => ctx.home.join(".claude"),
            Location::Local => ctx.cwd.join(".claude"),
        },
        AgentId::Cursor => match loc {
            Location::Global => ctx.home.join(".cursor"),
            Location::Local => ctx.cwd.join(".cursor"),
        },
        AgentId::Kiro => match loc {
            Location::Global => ctx.home.join(".kiro"),
            Location::Local => ctx.cwd.join(".kiro"),
        },
    };
    base.exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> InstallContext {
        InstallContext {
            home: PathBuf::from("/home/u"),
            cwd: PathBuf::from("/work/proj"),
            xdg_config_home: None,
        }
    }

    #[test]
    fn agent_id_round_trips_through_parse() {
        for agent in ALL_AGENTS {
            assert_eq!(AgentId::parse(agent.as_str()), Some(agent));
        }
        assert_eq!(AgentId::parse("OpenCode"), Some(AgentId::Opencode));
        assert_eq!(AgentId::parse("nope"), None);
    }

    #[test]
    fn opencode_uses_singular_skill_dir() {
        let global = skill_dir(AgentId::Opencode, &ctx(), Location::Global);
        assert!(
            global.ends_with("opencode/skill"),
            "expected opencode/skill, got {}",
            global.display()
        );
        let local = skill_dir(AgentId::Opencode, &ctx(), Location::Local);
        assert_eq!(local, PathBuf::from("/work/proj/.opencode/skill"));
    }

    #[test]
    fn opencode_global_honors_xdg_config_home() {
        let mut c = ctx();
        c.xdg_config_home = Some(PathBuf::from("/custom/xdg"));
        let global = skill_dir(AgentId::Opencode, &c, Location::Global);
        assert_eq!(global, PathBuf::from("/custom/xdg/opencode/skill"));
    }

    #[test]
    fn claude_cursor_kiro_use_plural_skills_dir() {
        assert_eq!(
            skill_dir(AgentId::Claude, &ctx(), Location::Local),
            PathBuf::from("/work/proj/.claude/skills")
        );
        assert_eq!(
            skill_dir(AgentId::Cursor, &ctx(), Location::Global),
            PathBuf::from("/home/u/.cursor/skills")
        );
        assert_eq!(
            skill_dir(AgentId::Kiro, &ctx(), Location::Global),
            PathBuf::from("/home/u/.kiro/skills")
        );
    }

    #[test]
    fn base_config_dir_detection_follows_home_and_cwd() {
        let base = std::env::temp_dir().join(format!(
            "scafgen-targets-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let home = base.join("home");
        let cwd = base.join("cwd");
        std::fs::create_dir_all(home.join(".claude")).unwrap();
        std::fs::create_dir_all(&cwd).unwrap();
        let c = InstallContext {
            home,
            cwd,
            xdg_config_home: None,
        };
        assert!(base_config_dir_exists(
            AgentId::Claude,
            &c,
            Location::Global
        ));
        assert!(!base_config_dir_exists(
            AgentId::Cursor,
            &c,
            Location::Global
        ));
        let _ = std::fs::remove_dir_all(&base);
    }
}
