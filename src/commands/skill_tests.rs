use super::*;
use std::path::PathBuf;

#[test]
fn resolve_targets_empty_is_all_four() {
    let all = resolve_targets(&[]).unwrap();
    assert_eq!(all, skill::ALL_AGENTS.to_vec());
}

#[test]
fn resolve_targets_subset_dedupes_and_orders() {
    let got = resolve_targets(&[
        "claude".to_string(),
        "opencode".to_string(),
        "claude".to_string(),
    ])
    .unwrap();
    assert_eq!(got, vec![AgentId::Opencode, AgentId::Claude]);
}

#[test]
fn resolve_targets_unknown_errors() {
    let err = resolve_targets(&["nope".to_string()]).unwrap_err();
    assert!(err.to_string().contains("unknown --target 'nope'"));
}

fn write_result_all_actions() -> WriteResult {
    WriteResult {
        files: vec![
            (PathBuf::from("/x/SKILL.md"), FileAction::Created),
            (PathBuf::from("/x/SKILL.md"), FileAction::Updated),
            (PathBuf::from("/x/SKILL.md"), FileAction::Removed),
            (PathBuf::from("/x/SKILL.md"), FileAction::Unchanged),
            (PathBuf::from("/x/SKILL.md"), FileAction::Skipped),
            (PathBuf::from("/x/SKILL.md"), FileAction::NotFound),
        ],
        notes: vec!["a note".to_string()],
    }
}

#[test]
fn report_write_handles_every_file_action_and_notes() {
    report_write(
        AgentId::Claude,
        Location::Global,
        "install",
        &write_result_all_actions(),
    );
}

#[test]
fn report_status_renders_each_status() {
    for status in [
        SkillStatus::NotInstalled,
        SkillStatus::UpToDate,
        SkillStatus::LocallyModified,
        SkillStatus::Outdated,
    ] {
        report_status(AgentId::Opencode, Location::Local, status);
    }
}

fn with_temp_home<F: FnOnce(&std::path::Path)>(f: F) {
    // 复用 completions_tests 里的同一把 HOME 改写锁，跨模块串行所有环境改写测试。
    let _guard = crate::commands::completions::tests::ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().expect("tempdir");
    let saved_home = std::env::var("HOME").ok();
    let saved_xdg = std::env::var("XDG_CONFIG_HOME").ok();
    // SAFETY: 持有 ENV_LOCK 串行执行，改写后立即在回调内使用并随后复原。
    unsafe {
        std::env::set_var("HOME", tmp.path());
        std::env::remove_var("XDG_CONFIG_HOME");
    }
    f(tmp.path());
    // SAFETY: 复原到进入前状态，仍在锁内。
    unsafe {
        match saved_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
        match saved_xdg {
            Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
            None => std::env::remove_var("XDG_CONFIG_HOME"),
        }
    }
}

#[test]
fn execute_status_without_target_warns_when_no_agent_dirs_exist() {
    with_temp_home(|_home| {
        let req = SkillRequest {
            action: Action::Status,
            targets: Vec::new(),
            local: false,
            force: false,
        };
        // 空 HOME 下无任何 agent 基准目录 → 命中 selected.is_empty() 早退分支。
        execute(req).expect("execute returns Ok on empty selection");
    });
}

#[test]
fn execute_install_explicit_target_writes_then_status_uptodate() {
    with_temp_home(|home| {
        let install = SkillRequest {
            action: Action::Install,
            targets: vec!["claude".to_string()],
            local: false,
            force: false,
        };
        execute(install).expect("install succeeds");
        let skill_md = home.join(".claude/skills/scaffold-gen/SKILL.md");
        assert!(skill_md.exists(), "SKILL.md installed at {skill_md:?}");

        let status = SkillRequest {
            action: Action::Status,
            targets: vec!["claude".to_string()],
            local: false,
            force: false,
        };
        execute(status).expect("status succeeds after install");
    });
}

#[test]
fn execute_update_and_uninstall_explicit_target() {
    with_temp_home(|home| {
        execute(SkillRequest {
            action: Action::Install,
            targets: vec!["cursor".to_string()],
            local: false,
            force: false,
        })
        .expect("install");
        execute(SkillRequest {
            action: Action::Update,
            targets: vec!["cursor".to_string()],
            local: false,
            force: true,
        })
        .expect("update --force");
        execute(SkillRequest {
            action: Action::Uninstall,
            targets: vec!["cursor".to_string()],
            local: false,
            force: false,
        })
        .expect("uninstall");
        let skill_md = home.join(".cursor/skills/scaffold-gen/SKILL.md");
        assert!(!skill_md.exists(), "SKILL.md removed after uninstall");
    });
}

#[test]
fn action_equality_is_derived() {
    assert_eq!(Action::Install, Action::Install);
    assert_ne!(Action::Install, Action::Status);
}
