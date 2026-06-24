use anyhow::{Context, Result};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;
use which::which;

/// 检查某个外部工具是否在 PATH 中可用。
///
/// 这是各生成器旧版 `check_pnpm` 风格探针的统一实现：仅判断可执行文件
/// 是否能在 PATH 解析到，不实际运行它。永不 panic、永不返回错误。
pub fn tool_available(name: &str) -> bool {
    which(name).is_ok()
}

/// 一次外部命令执行的结果，封装退出状态与捕获的输出。
#[derive(Debug, Clone)]
pub struct CommandOutcome {
    success: bool,
    stdout: String,
    stderr: String,
}

impl CommandOutcome {
    /// 命令是否以成功状态退出。
    pub fn success(&self) -> bool {
        self.success
    }

    /// 捕获的标准输出（lossy UTF-8）。
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    /// 捕获的标准错误（lossy UTF-8）。
    pub fn stderr(&self) -> &str {
        &self.stderr
    }
}

/// 外部命令的类型化构造器，集中处理「在某目录中运行 + 捕获输出 + 统一错误映射」。
///
/// 框架特定的「运行哪些命令、什么顺序」仍留在各生成器中；这里只负责运行机制。
#[derive(Debug, Clone)]
pub struct ExternalCommand {
    program: OsString,
    args: Vec<OsString>,
    cwd: Option<PathBuf>,
}

impl ExternalCommand {
    /// 以可执行文件名创建命令。
    pub fn new(program: impl AsRef<OsStr>) -> Self {
        Self {
            program: program.as_ref().to_os_string(),
            args: Vec::new(),
            cwd: None,
        }
    }

    /// 追加单个参数。
    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    /// 追加多个参数。
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for arg in args {
            self.args.push(arg.as_ref().to_os_string());
        }
        self
    }

    /// 设置命令运行的工作目录。
    pub fn current_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.cwd = Some(dir.as_ref().to_path_buf());
        self
    }

    /// 仅用于错误信息的「program args...」展示形式。
    fn display(&self) -> String {
        let mut parts = vec![self.program.to_string_lossy().into_owned()];
        parts.extend(self.args.iter().map(|a| a.to_string_lossy().into_owned()));
        parts.join(" ")
    }

    /// 运行命令并捕获输出。
    ///
    /// 仅当可执行文件无法被启动（如不存在）时返回 `Err`（带命令上下文，绝不 panic）；
    /// 非零退出状态被视为正常结果，由调用方通过 [`CommandOutcome::success`] 处理。
    pub fn run(&self) -> Result<CommandOutcome> {
        let mut command = Command::new(&self.program);
        command.args(&self.args);
        if let Some(dir) = &self.cwd {
            command.current_dir(dir);
        }

        let output = command
            .output()
            .with_context(|| format!("Failed to execute command: {}", self.display()))?;

        Ok(CommandOutcome {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_available_returns_false_for_absent_binary() {
        // 一个保证不存在的可执行文件名 —— 永远不应解析成功，且不能 panic。
        assert!(!tool_available("definitely-not-a-real-tool-xyz"));
    }

    #[test]
    fn run_maps_command_not_found_to_error_not_panic() {
        // 不存在的可执行文件无法启动 -> Err（不 panic）。
        let result = ExternalCommand::new("definitely-not-a-real-tool-xyz")
            .arg("--version")
            .run();
        assert!(result.is_err());
        let msg = format!("{:#}", result.unwrap_err());
        assert!(
            msg.contains("Failed to execute command"),
            "error context missing program display: {msg}"
        );
        assert!(
            msg.contains("definitely-not-a-real-tool-xyz"),
            "error context missing program name: {msg}"
        );
    }

    #[test]
    fn display_formats_program_and_args() {
        let cmd = ExternalCommand::new("pnpm")
            .args(["create", "vite@latest", "myapp"])
            .arg("--template")
            .arg("react-ts")
            .current_dir("/tmp");
        assert_eq!(
            cmd.display(),
            "pnpm create vite@latest myapp --template react-ts"
        );
    }

    #[test]
    fn builder_records_cwd_and_args() {
        let cmd = ExternalCommand::new("go")
            .args(["mod", "tidy"])
            .current_dir("/some/dir");
        assert_eq!(cmd.args, ["mod", "tidy"]);
        assert_eq!(cmd.cwd, Some(PathBuf::from("/some/dir")));
    }
}
