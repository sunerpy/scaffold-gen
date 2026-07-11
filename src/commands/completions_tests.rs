use super::*;
use clap::{Arg, Command as ClapCommand};

fn sample_cmd() -> ClapCommand {
    ClapCommand::new("scafgen")
        .subcommand(ClapCommand::new("new").arg(Arg::new("name")))
        .subcommand(ClapCommand::new("list"))
}

#[test]
fn bash_completion_is_non_empty_and_mentions_binary() {
    let mut cmd = sample_cmd();
    let mut buf = Vec::new();
    generate_to(Shell::Bash, &mut cmd, &mut buf);
    let script = String::from_utf8(buf).expect("utf8 completion");
    assert!(!script.is_empty());
    assert!(
        script.contains("scafgen"),
        "bash completion mentions scafgen"
    );
}

#[test]
fn zsh_completion_is_non_empty_and_mentions_binary() {
    let mut cmd = sample_cmd();
    let mut buf = Vec::new();
    generate_to(Shell::Zsh, &mut cmd, &mut buf);
    let script = String::from_utf8(buf).expect("utf8 completion");
    assert!(!script.is_empty());
    assert!(script.contains("scafgen"));
}

#[test]
fn completion_target_uses_binary_name() {
    if let Some(path) = completion_target(Shell::Fish) {
        assert!(path.to_string_lossy().contains("scafgen.fish"));
    }
}

#[test]
fn fish_and_powershell_and_elvish_completions_are_non_empty() {
    for shell in [Shell::Fish, Shell::PowerShell, Shell::Elvish] {
        let mut cmd = sample_cmd();
        let mut buf = Vec::new();
        generate_to(shell, &mut cmd, &mut buf);
        assert!(!buf.is_empty(), "{shell} completion must be non-empty");
    }
}

// 进程级环境是全局共享的；任何改写 HOME/XDG 的测试都必须串行，避免并行竞争。
// 该锁在整个 bin test crate 内共享（skill_tests 也引用它），是唯一的 HOME 改写门。
pub(crate) static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

type EnvGuard = (
    Vec<(&'static str, String)>,
    std::sync::MutexGuard<'static, ()>,
);

fn ctx_env(home: &str, xdg: Option<&str>, local: Option<&str>) -> EnvGuard {
    let guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut saved = Vec::new();
    for key in ["HOME", "USERPROFILE", "XDG_DATA_HOME", "LOCALAPPDATA"] {
        saved.push((key, std::env::var(key).unwrap_or_default()));
    }
    // SAFETY: 持有 ENV_LOCK 串行改写进程环境后立即读取并复原，仅本测试可见。
    unsafe {
        std::env::set_var("HOME", home);
        std::env::remove_var("USERPROFILE");
        match xdg {
            Some(v) => std::env::set_var("XDG_DATA_HOME", v),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
        match local {
            Some(v) => std::env::set_var("LOCALAPPDATA", v),
            None => std::env::remove_var("LOCALAPPDATA"),
        }
    }
    (saved, guard)
}

fn restore_env(guard: EnvGuard) {
    let (saved, _lock) = guard;
    // SAFETY: 仍持有 ENV_LOCK，复原测试前保存的环境变量后释放锁。
    unsafe {
        for (key, val) in saved {
            if val.is_empty() {
                std::env::remove_var(key);
            } else {
                std::env::set_var(key, val);
            }
        }
    }
}

#[test]
fn zsh_target_is_under_zfunc() {
    let saved = ctx_env("/home/tester", None, None);
    let path = completion_target(Shell::Zsh).expect("zsh target resolves with HOME");
    assert_eq!(path, PathBuf::from("/home/tester/.zfunc/_scafgen"));
    restore_env(saved);
}

#[test]
fn fish_target_is_under_config_fish() {
    let saved = ctx_env("/home/tester", None, None);
    let path = completion_target(Shell::Fish).expect("fish target resolves");
    assert_eq!(
        path,
        PathBuf::from("/home/tester/.config/fish/completions/scafgen.fish")
    );
    restore_env(saved);
}

#[test]
fn elvish_target_is_under_config_scafgen() {
    let saved = ctx_env("/home/tester", None, None);
    let path = completion_target(Shell::Elvish).expect("elvish target resolves");
    assert_eq!(
        path,
        PathBuf::from("/home/tester/.config/scafgen/completion.elv")
    );
    restore_env(saved);
}

#[test]
fn bash_target_prefers_xdg_data_home() {
    let saved = ctx_env("/home/tester", Some("/xdg/data"), None);
    let path = completion_target(Shell::Bash).expect("bash target resolves via XDG");
    assert_eq!(
        path,
        PathBuf::from("/xdg/data/bash-completion/completions/scafgen")
    );
    restore_env(saved);
}

#[test]
fn bash_target_falls_back_to_local_share() {
    let saved = ctx_env("/home/tester", None, None);
    let path = completion_target(Shell::Bash).expect("bash target resolves via HOME");
    assert_eq!(
        path,
        PathBuf::from("/home/tester/.local/share/bash-completion/completions/scafgen")
    );
    restore_env(saved);
}

#[test]
fn powershell_target_prefers_localappdata() {
    let saved = ctx_env("/home/tester", None, Some("/appdata/local"));
    let path = completion_target(Shell::PowerShell).expect("powershell target via LOCALAPPDATA");
    assert_eq!(path, PathBuf::from("/appdata/local/scafgen/completion.ps1"));
    restore_env(saved);
}

#[test]
fn env_path_treats_empty_as_absent() {
    let saved = ctx_env("/home/tester", Some(""), None);
    // XDG 为空串应被忽略，回退到 HOME/.local/share。
    let path = completion_target(Shell::Bash).expect("resolves");
    assert!(path.starts_with("/home/tester/.local/share"));
    restore_env(saved);
}

#[test]
fn write_completion_file_creates_parent_and_writes_bytes() {
    let dir = std::env::temp_dir().join(format!(
        "scafgen-comp-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let target = dir.join("nested/deep/scafgen.fish");
    write_completion_file(&target, b"# completion\n").expect("writes file + parents");
    assert_eq!(std::fs::read(&target).unwrap(), b"# completion\n");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn execute_print_mode_writes_script_to_stdout_without_error() {
    let mut cmd = sample_cmd();
    execute(Shell::Bash, false, &mut cmd).expect("print completion to stdout");
}

#[test]
fn post_install_hint_covers_every_shell_arm() {
    let target = PathBuf::from("/tmp/scafgen/completion");
    for shell in [
        Shell::Zsh,
        Shell::Elvish,
        Shell::PowerShell,
        Shell::Bash,
        Shell::Fish,
    ] {
        post_install_hint(shell, &target);
    }
}

#[test]
fn install_script_installs_into_resolved_target() {
    let saved = ctx_env(
        &std::env::temp_dir()
            .join(format!("scafgen-install-{}", std::process::id()))
            .to_string_lossy(),
        None,
        None,
    );
    let mut cmd = sample_cmd();
    install_script(Shell::Fish, &mut cmd).expect("install to resolved fish target");
    let expected = completion_target(Shell::Fish).expect("fish target resolves");
    assert!(expected.exists(), "completion written to {expected:?}");
    let _ = std::fs::remove_file(&expected);
    restore_env(saved);
}
