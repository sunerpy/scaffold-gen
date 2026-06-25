//! `scafgen list` 命令 —— 列出每种语言下可用的框架及其实现状态。
//!
//! 数据完全由 `registry::all_specs()` 驱动（与生成调度共享唯一真相来源），
//! 因此注册表增删一行，本命令输出自动跟随，无需改这里。机器可读输出走
//! `--json`；人类可读表格与 JSON 都打印到 STDOUT（可被管道消费）。

use anyhow::{Context, Result};

use crate::constants::{Framework, Language};
use crate::generators::registry::{self, FrameworkSpec, GenKind};

/// 一个框架在 `list` 中的展示状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// 已实现，可直接生成。
    Available,
    /// 枚举存在但生成器未实现（GoZero）。
    Unimplemented,
}

impl Status {
    fn from_kind(kind: GenKind) -> Self {
        match kind {
            GenKind::Unimplemented => Self::Unimplemented,
            GenKind::GinSync | GenKind::EmbeddedAsync | GenKind::ExternalAsync => Self::Available,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Unimplemented => "not implemented",
        }
    }

    fn json_value(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Unimplemented => "unimplemented",
        }
    }
}

/// 一行可展示的框架信息（语言 + 框架 CLI 名 + 显示名 + 状态）。
#[derive(Debug, Clone)]
pub struct Entry {
    pub language: Language,
    pub framework: Framework,
    pub display_name: &'static str,
    pub status: Status,
}

impl Entry {
    fn from_spec(spec: &FrameworkSpec) -> Self {
        Self {
            language: spec.language,
            framework: spec.framework,
            display_name: spec.framework.display_name(),
            status: Status::from_kind(spec.kind),
        }
    }
}

/// 收集全部展示条目（按 `registry::all_specs` 的稳定顺序）。
pub fn entries() -> Vec<Entry> {
    registry::all_specs().iter().map(Entry::from_spec).collect()
}

/// 渲染人类可读的、按语言分组的表格文本。
pub fn render_table(entries: &[Entry]) -> String {
    let langs = [
        Language::Go,
        Language::Python,
        Language::Rust,
        Language::TypeScript,
    ];
    let mut out = String::from("Available languages and frameworks:\n");
    for lang in langs {
        let group: Vec<&Entry> = entries.iter().filter(|e| e.language == lang).collect();
        if group.is_empty() {
            continue;
        }
        out.push_str(&format!("\n{lang}:\n"));
        for entry in group {
            let name = entry.framework.as_str();
            out.push_str(&format!(
                "  {name:<12} {} [{}]\n",
                entry.display_name,
                entry.status.label()
            ));
        }
    }
    out
}

/// 渲染机器可读的 JSON 文本。
pub fn render_json(entries: &[Entry]) -> Result<String> {
    let items: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "language": e.language.as_str(),
                "framework": e.framework.as_str(),
                "displayName": e.display_name,
                "status": e.status.json_value(),
            })
        })
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({ "frameworks": items }))
        .context("serializing framework list to JSON")
}

/// `scafgen list` 入口。`json=true` 时输出 JSON，否则输出分组表格。两者均写 STDOUT。
pub fn execute(json: bool) -> Result<()> {
    let entries = entries();
    if json {
        println!("{}", render_json(&entries)?);
    } else {
        print!("{}", render_table(&entries));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_cover_every_framework() {
        let entries = entries();
        for fw in [
            Framework::Gin,
            Framework::GoZero,
            Framework::McpServer,
            Framework::FastApi,
            Framework::Tauri,
            Framework::Vue3,
            Framework::React,
        ] {
            assert!(
                entries.iter().any(|e| e.framework == fw),
                "list entries missing framework {fw:?}"
            );
        }
    }

    #[test]
    fn gozero_is_marked_unimplemented() {
        let entries = entries();
        let gozero = entries
            .iter()
            .find(|e| e.framework == Framework::GoZero)
            .expect("gozero entry present");
        assert_eq!(gozero.status, Status::Unimplemented);
    }

    #[test]
    fn table_lists_all_frameworks_and_flags_gozero() {
        let table = render_table(&entries());
        for fw in [
            Framework::Gin,
            Framework::GoZero,
            Framework::McpServer,
            Framework::FastApi,
            Framework::Tauri,
            Framework::Vue3,
            Framework::React,
        ] {
            assert!(
                table.contains(fw.as_str()),
                "table missing framework {fw:?}: {table}"
            );
        }
        assert!(table.contains("fastapi"));
        assert!(table.contains("mcp-server"));
        assert!(table.contains("go-zero"));
        assert!(table.contains("not implemented"));
    }

    #[test]
    fn json_is_valid_and_marks_gozero_unimplemented() {
        let json = render_json(&entries()).expect("json renders");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let arr = parsed["frameworks"].as_array().expect("frameworks array");
        assert!(arr.iter().any(|e| e["framework"] == "fastapi"));
        assert!(arr.iter().any(|e| e["framework"] == "mcp-server"));
        let gozero = arr
            .iter()
            .find(|e| e["framework"] == "go-zero")
            .expect("go-zero in json");
        assert_eq!(gozero["status"], "unimplemented");
    }
}
