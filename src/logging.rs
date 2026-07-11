//! 结构化日志初始化层。
//!
//! 用 `tracing` 取代散落各处的 `println!`/`eprintln!`，由 CLI 的 `-q/--quiet` 与
//! `-v/--verbose` 全局开关控制详细程度，并支持 `RUST_LOG` 环境变量覆盖。
//!
//! 这是一个用户 CLI（而非服务端日志），因此 `fmt` 订阅器关闭了时间戳、目标、级别
//! 等噪声前缀，使默认级别下的输出尽量贴近原先干净的人类可读信息。

use tracing_subscriber::EnvFilter;

/// CLI 详细程度级别 —— 由全局 `-q`/`-v` 开关解析得到。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    /// `--quiet`：仅错误。
    Quiet,
    /// 默认：关键里程碑 + 后续步骤（info）。
    Normal,
    /// `--verbose`：包含进度细节（debug）。
    Verbose,
}

impl Verbosity {
    /// 由 `quiet`/`verbose` 两个布尔开关推导级别。`quiet` 优先于 `verbose`。
    pub fn from_flags(quiet: bool, verbose: bool) -> Self {
        if quiet {
            Self::Quiet
        } else if verbose {
            Self::Verbose
        } else {
            Self::Normal
        }
    }

    /// 映射为 tracing 的级别过滤指令字符串。
    fn default_directive(self) -> &'static str {
        match self {
            Self::Quiet => "error",
            Self::Normal => "info",
            Self::Verbose => "debug",
        }
    }
}

/// 初始化全局 tracing 订阅器（进程内只应调用一次，在命令分发之前）。
///
/// 过滤级别由 `verbosity` 决定，但当设置了 `RUST_LOG` 环境变量时以它为准。
pub fn init(verbosity: Verbosity) {
    // RUST_LOG 覆盖；否则用开关推导的级别。
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(verbosity.default_directive()));

    // 用户 CLI：关闭时间戳/目标/级别前缀以贴近原先 println 风格；写 stderr 以保持
    // stdout 纯净（为未来 --json 留出空间）并保留原 `eprintln!` 错误进 stderr 的契约。
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_level(false)
        .without_time()
        .with_writer(std::io::stderr)
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_takes_priority_over_verbose() {
        assert_eq!(Verbosity::from_flags(true, true), Verbosity::Quiet);
        assert_eq!(Verbosity::from_flags(true, false), Verbosity::Quiet);
    }

    #[test]
    fn verbose_flag_maps_to_verbose() {
        assert_eq!(Verbosity::from_flags(false, true), Verbosity::Verbose);
    }

    #[test]
    fn no_flags_maps_to_normal() {
        assert_eq!(Verbosity::from_flags(false, false), Verbosity::Normal);
    }

    #[test]
    fn directives_match_levels() {
        assert_eq!(Verbosity::Quiet.default_directive(), "error");
        assert_eq!(Verbosity::Normal.default_directive(), "info");
        assert_eq!(Verbosity::Verbose.default_directive(), "debug");
    }

    #[test]
    fn init_is_idempotent_and_never_panics() {
        // try_init 幂等：进程内可能已有全局订阅器，二次调用返回 Err 被吞掉，
        // 覆盖 init() 的 filter 构建 + fmt 订阅器构建路径而不 panic。
        init(Verbosity::Normal);
        init(Verbosity::Verbose);
        init(Verbosity::Quiet);
    }
}
