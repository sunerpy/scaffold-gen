//! 嵌入的 SKILL.md + git-blob 哈希 + sidecar 标记。
//!
//! 在编译期把 `skills/scaffold-gen/SKILL.md` 内联进二进制，并提供更新判定所
//! 依赖的 git-blob SHA-1（与 `git hash-object` 一致）。sidecar 标记保存
//! `{hash, version, installed_at}`，其中 hash 是唯一的更新判定输入，version /
//! installed_at 仅供人类参考。

use serde::{Deserialize, Serialize};

/// 编译期内联的 scaffold-gen 技能内容。
///
/// 路径相对于本文件（`src/skill/embed.rs`）回到仓库根：`embed.rs` → `skill`
/// → `src` → 仓库根，共两次 `../`，再进入 `skills/scaffold-gen/SKILL.md`。
pub const SKILL_MD: &str = include_str!("../../skills/scaffold-gen/SKILL.md");

/// 技能目录名（与 SKILL.md frontmatter 的 `name:` 一致）。
pub const SKILL_DIR_NAME: &str = "scaffold-gen";

/// 技能文件名（大小写敏感，遵循 Open Agent Skills 规范）。
pub const SKILL_FILE_NAME: &str = "SKILL.md";

/// 写在技能旁的 sidecar 标记文件名。
pub const SIDECAR_FILE_NAME: &str = ".scaffold-gen-skill.json";

/// 计算 `content` 的 git blob 对象哈希。
///
/// Git 把 blob 哈希为 `sha1("blob " + len + "\0" + content)`，返回小写十六进制
/// 摘要。这是 SHA-1（非 SHA-256），与 `git hash-object` 完全一致；空 blob 即
/// 众所周知的 `e69de29bb2d1d6434b8b29ae775ad8c2e48c5391`。
pub fn git_blob_sha1(content: &[u8]) -> String {
    let mut hasher = sha1_smol::Sha1::new();
    let header = format!("blob {}\0", content.len());
    hasher.update(header.as_bytes());
    hasher.update(content);
    hasher.digest().to_string()
}

/// 持久化在 `<skill-dir>/.scaffold-gen-skill.json` 的 sidecar 标记。
///
/// `hash` 是所写 SKILL.md 内容的 git-blob SHA-1，是更新判定的唯一输入；
/// `version` / `installed_at` 仅为人类可读的来源信息，不参与判定。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillMarker {
    /// 本标记伴随的 SKILL.md 内容的 git-blob SHA-1。
    pub hash: String,
    /// 写入技能时的 CLI 版本（信息性）。
    pub version: String,
    /// 写入技能的 RFC3339 时间戳（信息性）。
    pub installed_at: String,
}

impl SkillMarker {
    /// 为当前嵌入内容在当下时刻构造一份新标记。
    pub fn for_embedded() -> Self {
        Self {
            hash: git_blob_sha1(SKILL_MD.as_bytes()),
            version: env!("CARGO_PKG_VERSION").to_string(),
            installed_at: now_rfc3339(),
        }
    }

    /// 序列化为带尾换行的 pretty JSON；序列化理论上不会失败，失败则回退到手写对象。
    pub fn to_pretty_json(&self) -> String {
        let mut content = serde_json::to_string_pretty(self).unwrap_or_else(|_| {
            format!(
                "{{\n  \"hash\": \"{}\",\n  \"version\": \"{}\",\n  \"installed_at\": \"{}\"\n}}",
                self.hash, self.version, self.installed_at
            )
        });
        content.push('\n');
        content
    }
}

/// 当前 UTC 时间的 RFC3339 字符串（复用项目已有的 `chrono` 依赖）。
fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_blob_sha1_empty_matches_git() {
        // `printf '' | git hash-object --stdin`
        assert_eq!(
            git_blob_sha1(b""),
            "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
        );
    }

    #[test]
    fn git_blob_sha1_hello_matches_git() {
        // `printf 'hello\n' | git hash-object --stdin`
        assert_eq!(
            git_blob_sha1(b"hello\n"),
            "ce013625030ba8dba906f756967f9e9ca394464a"
        );
    }

    #[test]
    fn skill_md_is_embedded_and_well_formed() {
        assert!(SKILL_MD.starts_with("---\n"), "must start with YAML fence");
        assert!(
            SKILL_MD.contains("name: scaffold-gen"),
            "must declare the scaffold-gen skill name"
        );
    }

    #[test]
    fn marker_for_embedded_records_embedded_hash_and_version() {
        let marker = SkillMarker::for_embedded();
        assert_eq!(marker.hash, git_blob_sha1(SKILL_MD.as_bytes()));
        assert_eq!(marker.version, env!("CARGO_PKG_VERSION"));
        assert!(!marker.installed_at.is_empty());
    }

    #[test]
    fn marker_pretty_json_round_trips() {
        let marker = SkillMarker::for_embedded();
        let json = marker.to_pretty_json();
        assert!(json.ends_with('\n'));
        let parsed: SkillMarker = serde_json::from_str(&json).expect("round-trips");
        assert_eq!(parsed, marker);
    }
}
