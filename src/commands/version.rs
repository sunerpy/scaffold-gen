//! `scafgen version` 命令 —— 打印名称、版本与仓库地址到 STDOUT。
//!
//! 比 `--version` 更丰富：附带 crate 名与仓库 URL，便于用户/agent 报告环境。
//! 版本来自编译期 `CARGO_PKG_*`，与 `--version` 同源、永不漂移。

/// 打印版本信息到 STDOUT（机器/人类皆可读的纯数据输出）。
pub fn execute() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("repository: {}", env!("CARGO_PKG_REPOSITORY"));
}
