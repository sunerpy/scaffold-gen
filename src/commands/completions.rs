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
}
