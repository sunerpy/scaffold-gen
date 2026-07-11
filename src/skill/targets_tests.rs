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
fn location_as_str_maps_both_variants() {
    assert_eq!(Location::Global.as_str(), "global");
    assert_eq!(Location::Local.as_str(), "local");
}

#[test]
fn agent_display_names_are_human_readable() {
    assert_eq!(AgentId::Opencode.display_name(), "opencode");
    assert_eq!(AgentId::Claude.display_name(), "Claude Code");
    assert_eq!(AgentId::Cursor.display_name(), "Cursor");
    assert_eq!(AgentId::Kiro.display_name(), "Kiro");
}

#[test]
fn opencode_global_falls_back_to_home_dot_config_without_xdg() {
    let global = skill_dir(AgentId::Opencode, &ctx(), Location::Global);
    assert_eq!(global, PathBuf::from("/home/u/.config/opencode/skill"));
}

#[test]
fn claude_cursor_kiro_global_and_local_dirs_are_plural() {
    assert_eq!(
        skill_dir(AgentId::Claude, &ctx(), Location::Global),
        PathBuf::from("/home/u/.claude/skills")
    );
    assert_eq!(
        skill_dir(AgentId::Cursor, &ctx(), Location::Local),
        PathBuf::from("/work/proj/.cursor/skills")
    );
    assert_eq!(
        skill_dir(AgentId::Kiro, &ctx(), Location::Local),
        PathBuf::from("/work/proj/.kiro/skills")
    );
}

#[test]
fn base_config_dir_exists_covers_every_agent_and_location() {
    let base = std::env::temp_dir().join(format!(
        "scafgen-targets-all-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let home = base.join("home");
    let cwd = base.join("cwd");
    std::fs::create_dir_all(home.join(".config").join("opencode")).unwrap();
    std::fs::create_dir_all(home.join(".claude")).unwrap();
    std::fs::create_dir_all(home.join(".cursor")).unwrap();
    std::fs::create_dir_all(home.join(".kiro")).unwrap();
    std::fs::create_dir_all(cwd.join(".opencode")).unwrap();
    std::fs::create_dir_all(cwd.join(".claude")).unwrap();
    std::fs::create_dir_all(cwd.join(".cursor")).unwrap();
    std::fs::create_dir_all(cwd.join(".kiro")).unwrap();
    let c = InstallContext {
        home,
        cwd,
        xdg_config_home: None,
    };
    for agent in ALL_AGENTS {
        assert!(
            base_config_dir_exists(agent, &c, Location::Global),
            "global base dir should exist for {}",
            agent.as_str()
        );
        assert!(
            base_config_dir_exists(agent, &c, Location::Local),
            "local base dir should exist for {}",
            agent.as_str()
        );
    }
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn from_env_reads_home_cwd_and_optional_xdg() {
    let ctx = InstallContext::from_env().expect("resolve from env");
    assert!(!ctx.home.as_os_str().is_empty());
    assert!(ctx.cwd.is_absolute());
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
