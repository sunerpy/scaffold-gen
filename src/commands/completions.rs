//! `scafgen completions <SHELL>` 命令 —— 生成 shell 补全脚本。
//!
//! 默认把脚本打印到 STDOUT（必须是纯 stdout，使 `eval "$(scafgen completions zsh)"`
//! 可用）；`--install` 则尽力写入对应 shell 的标准补全目录，并把安装进度/提示走
//! tracing(stderr)。命令名固定为 `scafgen`（与 `[[bin]]` 一致）。

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Command;
use clap_complete::{Shell, generate};

const BIN_NAME: &str = "scafgen";

/// 把补全脚本写入 `buf`，返回字节。命令名固定 `scafgen`。
pub fn generate_to(shell: Shell, cmd: &mut Command, buf: &mut Vec<u8>) {
    generate(shell, cmd, BIN_NAME, buf);
}

/// `scafgen completions` 入口。`install=false` → 打印到 STDOUT；`true` → 尽力安装。
pub fn execute(shell: Shell, install: bool, cmd: &mut Command) -> Result<()> {
    if install {
        return install_script(shell, cmd);
    }
    let mut buf = Vec::new();
    generate_to(shell, cmd, &mut buf);
    let mut stdout = std::io::stdout();
    use std::io::Write;
    stdout
        .write_all(&buf)
        .context("writing completion script to stdout")?;
    Ok(())
}

fn install_script(shell: Shell, cmd: &mut Command) -> Result<()> {
    let target = match completion_target(shell) {
        Some(path) => path,
        None => {
            let mut buf = Vec::new();
            generate_to(shell, cmd, &mut buf);
            let script = String::from_utf8_lossy(&buf);
            tracing::warn!(
                "Cannot determine a standard completion location for {shell}. \
                 Generate it with `scafgen completions {shell}` and place it where your \
                 shell loads completions from."
            );
            print!("{script}");
            return Ok(());
        }
    };

    let mut buf = Vec::new();
    generate_to(shell, cmd, &mut buf);
    write_completion_file(&target, &buf)?;
    tracing::info!("Installed {shell} completions to {}", target.display());
    post_install_hint(shell, &target);
    Ok(())
}

fn post_install_hint(shell: Shell, target: &Path) {
    match shell {
        Shell::Zsh => {
            tracing::info!(
                "Add `fpath+=~/.zfunc` before `compinit` in your ~/.zshrc if not already present, \
                 then restart your shell."
            );
        }
        Shell::Elvish => {
            tracing::info!(
                "Add `eval (slurp < {})` to your ~/.config/elvish/rc.elv to load completions.",
                target.display()
            );
        }
        Shell::PowerShell => {
            tracing::info!(
                "Dot-source {} from your PowerShell $PROFILE to load completions.",
                target.display()
            );
        }
        _ => {
            tracing::info!("Restart your shell to load completions.");
        }
    }
}

fn write_completion_file(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating completion directory {}", parent.display()))?;
    }
    std::fs::write(path, bytes)
        .with_context(|| format!("writing completion script {}", path.display()))?;
    Ok(())
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

fn home_dir() -> Option<PathBuf> {
    env_path("HOME").or_else(|| env_path("USERPROFILE"))
}

/// 解析各 shell 的标准补全文件位置；无法可靠确定时返回 `None`。
fn completion_target(shell: Shell) -> Option<PathBuf> {
    match shell {
        Shell::Bash => {
            let base =
                env_path("XDG_DATA_HOME").or_else(|| home_dir().map(|h| h.join(".local/share")))?;
            Some(base.join("bash-completion/completions/scafgen"))
        }
        Shell::Zsh => Some(home_dir()?.join(".zfunc/_scafgen")),
        Shell::Fish => Some(home_dir()?.join(".config/fish/completions/scafgen.fish")),
        Shell::Elvish => Some(home_dir()?.join(".config/scafgen/completion.elv")),
        Shell::PowerShell => {
            let base =
                env_path("LOCALAPPDATA").or_else(|| home_dir().map(|h| h.join(".local/share")))?;
            Some(base.join("scafgen/completion.ps1"))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
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
        let path =
            completion_target(Shell::PowerShell).expect("powershell target via LOCALAPPDATA");
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
}
