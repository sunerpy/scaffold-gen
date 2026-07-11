//! mcp-python 后端选择 —— `McpBackend` 枚举。
//!
//! 决定生成的 mcp-python 项目所针对的 Python MCP 库：
//! - `Fastmcp`：独立 fastmcp（jlowin/PrefectHQ），锁定 `>=2,<3`，默认。
//! - `Official`：官方 modelcontextprotocol Python SDK（`mcp[cli]`），锁定 v1 线。
//!
//! 每个后端的精确 Python API 契约（import / app-factory / 内存测试客户端 /
//! 依赖 pin）记录在 `.omo/notepads/mcp-python-scaffold/learnings.md` 的
//! "Todo 0 — VERIFIED API CONTRACT" 一节，下游模板照此实现，避免双后端 API 漂移。

use serde::{Deserialize, Serialize};

/// Which Python MCP library the generated mcp-python project targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum McpBackend {
    /// Standalone fastmcp (jlowin/PrefectHQ), pinned >=2,<3. Default.
    #[default]
    Fastmcp,
    /// Official modelcontextprotocol Python SDK (mcp[cli]), pinned v1 line.
    Official,
}

impl McpBackend {
    /// 后端的稳定字符串标识（写入 config.toml `[mcp] backend`、模板上下文等）。
    pub fn as_str(&self) -> &'static str {
        match self {
            McpBackend::Fastmcp => "fastmcp",
            McpBackend::Official => "official",
        }
    }

    /// 是否为官方 SDK 后端 —— 模板用 `<%if mcp_backend_is_official%>` 分支。
    pub fn is_official(&self) -> bool {
        matches!(self, McpBackend::Official)
    }

    /// 从字符串解析后端；大小写不敏感。`"mcp"` 作为 `official` 的别名。
    pub fn parse_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fastmcp" => Some(McpBackend::Fastmcp),
            "official" | "mcp" => Some(McpBackend::Official),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "mcp_options_tests.rs"]
mod tests;
