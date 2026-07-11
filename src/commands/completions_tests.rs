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

fn ctx_env(home: &str, xdg: Option<&str>, local: Option<&str>) -> Vec<(&'static str, String)> {
    let mut saved = Vec::new();
    for key in ["HOME", "USERPROFILE", "XDG_DATA_HOME", "LOCALAPPDATA"] {
        saved.push((key, std::env::var(key).unwrap_or_default()));
    }
    // SAFETY: 测试串行改写进程环境后立即读取并复原，仅本测试可见。
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
    saved
}

fn restore_env(saved: Vec<(&'static str, String)>) {
    // SAFETY: 复原测试前保存的环境变量，串行执行。
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
