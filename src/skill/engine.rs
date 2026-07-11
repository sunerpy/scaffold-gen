//! 单技能目录的写入 / 更新 / 卸载 / 状态引擎。
//!
//! 所有函数都接收一个“技能父目录”（最终包含 `scaffold-gen/` 技能文件夹的目录），
//! 自行拥有全部文件系统读写与判定逻辑。更新判定仅由 git-blob SHA-1 驱动
//! （embedded vs 已安装内容），sidecar 仅提供来源信息以区分“用户改过”与“嵌入变了”。

use std::fs;
use std::path::{Path, PathBuf};

use super::embed::{
    SIDECAR_FILE_NAME, SKILL_DIR_NAME, SKILL_FILE_NAME, SKILL_MD, SkillMarker, git_blob_sha1,
};

/// 单个文件被引擎做了什么动作（驱动逐项报告）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAction {
    /// 新建（之前不存在）。
    Created,
    /// 覆盖更新（之前存在）。
    Updated,
    /// 内容与嵌入一致，未写入。
    Unchanged,
    /// 被本地修改，未写入（除非 --force）。
    Skipped,
    /// 已删除。
    Removed,
    /// 卸载时未找到可删的文件。
    NotFound,
}

impl FileAction {
    /// 人类可读的动词。
    pub fn verb(self) -> &'static str {
        match self {
            FileAction::Created => "created",
            FileAction::Updated => "updated",
            FileAction::Unchanged => "unchanged",
            FileAction::Skipped => "skipped (locally modified)",
            FileAction::Removed => "removed",
            FileAction::NotFound => "not found",
        }
    }
}

/// 引擎对一个技能目录采取动作后的结果。
#[derive(Debug, Clone, Default)]
pub struct WriteResult {
    /// 触及的文件及其动作。
    pub files: Vec<(PathBuf, FileAction)>,
    /// 附加说明（如本地修改提示、I/O 失败原因）。
    pub notes: Vec<String>,
}

/// 引擎对单个技能目录得出的更新决策。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateDecision {
    /// 已安装内容等于嵌入技能 —— 不写。
    Unchanged,
    /// 应（覆盖）写入：全新安装、强制、或来源确认过的更新。
    Update,
    /// 已安装内容不同且非我们所写 —— 保持原样。
    LocallyModified,
}

/// 一个技能目录的安装状态（供 `status` 渲染）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillStatus {
    /// 技能目录中没有 SKILL.md。
    NotInstalled,
    /// 已安装内容等于嵌入技能。
    UpToDate,
    /// 已安装内容不同，且非我们所写（用户改过）。
    LocallyModified,
    /// 已安装内容不同，但与 sidecar 记录一致（一次 install 会刷新到当前版本）。
    Outdated,
}

impl SkillStatus {
    /// 人类可读标签。
    pub fn label(self) -> &'static str {
        match self {
            SkillStatus::NotInstalled => "not installed",
            SkillStatus::UpToDate => "installed (up to date)",
            SkillStatus::LocallyModified => "installed (locally modified)",
            SkillStatus::Outdated => "installed (update available)",
        }
    }
}

/// 判定对单个技能目录应采取什么动作。
///
/// `installed_content` 为当前磁盘上的 SKILL.md（None ⇒ 未安装）；`sidecar` 为已
/// 解析的标记（若有）；`force` 覆盖本地修改保护。穷尽分支表：
///
/// 1. `force && installed.is_some()` → `Update`
/// 2. `installed.is_none()` → `Update`（全新安装）
/// 3. `installed == embedded`（按 git-blob SHA-1）→ `Unchanged`
/// 4. 漂移 + `sidecar.hash == sha(installed)` → `Update`（我们写的，刷新）
/// 5. 漂移 + `sidecar.hash != sha(installed)` → `LocallyModified`
/// 6. 漂移 + 无 sidecar → `LocallyModified`（保守，来源未知）
pub fn decide(
    installed_content: Option<&str>,
    sidecar: Option<&SkillMarker>,
    force: bool,
) -> UpdateDecision {
    let Some(installed) = installed_content else {
        return UpdateDecision::Update;
    };
    if force {
        return UpdateDecision::Update;
    }
    let embedded_hash = git_blob_sha1(SKILL_MD.as_bytes());
    let installed_hash = git_blob_sha1(installed.as_bytes());
    if installed_hash == embedded_hash {
        return UpdateDecision::Unchanged;
    }
    match sidecar {
        Some(marker) if marker.hash == installed_hash => UpdateDecision::Update,
        _ => UpdateDecision::LocallyModified,
    }
}

/// `<parent>/scaffold-gen` —— 给定父目录下的技能文件夹。
fn skill_dir(skill_parent_dir: &Path) -> PathBuf {
    skill_parent_dir.join(SKILL_DIR_NAME)
}

/// 给定父目录下的 SKILL.md 路径。
fn skill_file(skill_parent_dir: &Path) -> PathBuf {
    skill_dir(skill_parent_dir).join(SKILL_FILE_NAME)
}

/// 给定父目录下的 sidecar 标记路径。
fn sidecar_file(skill_parent_dir: &Path) -> PathBuf {
    skill_dir(skill_parent_dir).join(SIDECAR_FILE_NAME)
}

/// 原子写文件：先写 `<path>.tmp.<pid>`，再 rename；自动创建父目录。
fn atomic_write_file(path: &Path, content: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent()
        && !dir.as_os_str().is_empty()
    {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
    match fs::write(&tmp, content).and_then(|()| fs::rename(&tmp, path)) {
        Ok(()) => Ok(()),
        Err(err) => {
            let _ = fs::remove_file(&tmp);
            Err(err)
        }
    }
}

/// 读取已安装的 SKILL.md 内容与已解析 sidecar（若存在）。
///
/// 存在但无法解析的 sidecar 读作 `None`（视为来源缺失，`decide` 据此保守处理）。
pub fn read_installed(skill_parent_dir: &Path) -> (Option<String>, Option<SkillMarker>) {
    let content = fs::read_to_string(skill_file(skill_parent_dir)).ok();
    let sidecar = fs::read_to_string(sidecar_file(skill_parent_dir))
        .ok()
        .and_then(|text| serde_json::from_str::<SkillMarker>(&text).ok());
    (content, sidecar)
}

/// 把嵌入技能写入 `<skill_parent_dir>/scaffold-gen/`。
///
/// 读取既有内容 + sidecar，运行 [`decide`]，据此：Unchanged 不写；LocallyModified
/// （且非 force）不写并附说明；Update/全新/force 原子写 SKILL.md + 刷新 sidecar。
/// I/O 失败时返回 Skipped + 说明，而非 panic。
pub fn write_skill_to_dir(skill_parent_dir: &Path, force: bool) -> WriteResult {
    let skill_path = skill_file(skill_parent_dir);
    let sidecar_path = sidecar_file(skill_parent_dir);
    let (installed, sidecar) = read_installed(skill_parent_dir);
    let existed = installed.is_some();

    match decide(installed.as_deref(), sidecar.as_ref(), force) {
        UpdateDecision::Unchanged => WriteResult {
            files: vec![(skill_path, FileAction::Unchanged)],
            notes: Vec::new(),
        },
        UpdateDecision::LocallyModified => WriteResult {
            files: vec![(skill_path, FileAction::Skipped)],
            notes: vec![
                "skill locally modified — left unchanged (use --force to overwrite)".to_string(),
            ],
        },
        UpdateDecision::Update => {
            let marker = SkillMarker::for_embedded();
            if let Err(err) = atomic_write_file(&skill_path, SKILL_MD) {
                return WriteResult {
                    files: vec![(skill_path, FileAction::Skipped)],
                    notes: vec![format!("failed to write skill: {err}")],
                };
            }
            let action = if existed {
                FileAction::Updated
            } else {
                FileAction::Created
            };
            if let Err(err) = atomic_write_file(&sidecar_path, &marker.to_pretty_json()) {
                return WriteResult {
                    files: vec![(skill_path, action)],
                    notes: vec![format!("wrote skill but failed to write marker: {err}")],
                };
            }
            WriteResult {
                files: vec![(skill_path, action), (sidecar_path, action)],
                notes: Vec::new(),
            }
        }
    }
}

/// 从 `<skill_parent_dir>/scaffold-gen/` 移除技能。
///
/// 删除 SKILL.md + sidecar，再删除已空的 `scaffold-gen/` 目录。逐个删除的文件报
/// `Removed`；若本无安装则返回单个 `NotFound`。
pub fn uninstall_from_dir(skill_parent_dir: &Path) -> WriteResult {
    let dir = skill_dir(skill_parent_dir);
    let skill_path = skill_file(skill_parent_dir);
    let sidecar_path = sidecar_file(skill_parent_dir);

    let mut files = Vec::new();
    if skill_path.exists() && fs::remove_file(&skill_path).is_ok() {
        files.push((skill_path.clone(), FileAction::Removed));
    }
    if sidecar_path.exists() && fs::remove_file(&sidecar_path).is_ok() {
        files.push((sidecar_path, FileAction::Removed));
    }

    if files.is_empty() {
        return WriteResult {
            files: vec![(skill_path, FileAction::NotFound)],
            notes: Vec::new(),
        };
    }

    // 尽力删除已空的技能目录；用户额外放了文件导致非空时忽略失败。
    let _ = fs::remove_dir(&dir);
    WriteResult {
        files,
        notes: Vec::new(),
    }
}

/// 报告单个技能目录的安装状态。
pub fn status_for_dir(skill_parent_dir: &Path) -> SkillStatus {
    let (installed, sidecar) = read_installed(skill_parent_dir);
    let Some(installed) = installed else {
        return SkillStatus::NotInstalled;
    };
    let embedded_hash = git_blob_sha1(SKILL_MD.as_bytes());
    let installed_hash = git_blob_sha1(installed.as_bytes());
    if installed_hash == embedded_hash {
        return SkillStatus::UpToDate;
    }
    match sidecar {
        Some(marker) if marker.hash == installed_hash => SkillStatus::Outdated,
        _ => SkillStatus::LocallyModified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_parent(label: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "scafgen-skill-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn file_action_verbs_cover_every_variant() {
        assert_eq!(FileAction::Created.verb(), "created");
        assert_eq!(FileAction::Updated.verb(), "updated");
        assert_eq!(FileAction::Unchanged.verb(), "unchanged");
        assert_eq!(FileAction::Skipped.verb(), "skipped (locally modified)");
        assert_eq!(FileAction::Removed.verb(), "removed");
        assert_eq!(FileAction::NotFound.verb(), "not found");
    }

    #[test]
    fn skill_status_labels_cover_every_variant() {
        assert_eq!(SkillStatus::NotInstalled.label(), "not installed");
        assert_eq!(SkillStatus::UpToDate.label(), "installed (up to date)");
        assert_eq!(
            SkillStatus::LocallyModified.label(),
            "installed (locally modified)"
        );
        assert_eq!(
            SkillStatus::Outdated.label(),
            "installed (update available)"
        );
    }

    #[test]
    fn decide_fresh_install_is_update() {
        assert_eq!(decide(None, None, false), UpdateDecision::Update);
        assert_eq!(decide(None, None, true), UpdateDecision::Update);
    }

    #[test]
    fn decide_force_overwrites_existing() {
        assert_eq!(decide(Some("anything"), None, true), UpdateDecision::Update);
    }

    #[test]
    fn decide_identical_is_unchanged() {
        assert_eq!(
            decide(Some(SKILL_MD), None, false),
            UpdateDecision::Unchanged
        );
    }

    #[test]
    fn decide_drift_matching_sidecar_is_update() {
        let installed = "we wrote this once\n";
        let marker = SkillMarker {
            hash: git_blob_sha1(installed.as_bytes()),
            version: "0.1.0".into(),
            installed_at: "t".into(),
        };
        assert_eq!(
            decide(Some(installed), Some(&marker), false),
            UpdateDecision::Update
        );
    }

    #[test]
    fn decide_drift_mismatching_sidecar_is_locally_modified() {
        let marker = SkillMarker {
            hash: git_blob_sha1(b"some other content"),
            version: "0.1.0".into(),
            installed_at: "t".into(),
        };
        assert_eq!(
            decide(Some("user edited this\n"), Some(&marker), false),
            UpdateDecision::LocallyModified
        );
    }

    #[test]
    fn decide_drift_no_sidecar_is_locally_modified() {
        assert_eq!(
            decide(Some("mystery content\n"), None, false),
            UpdateDecision::LocallyModified
        );
    }

    #[test]
    fn write_creates_skill_and_sidecar_with_embedded_content() {
        let parent = temp_parent("write-create");
        let r = write_skill_to_dir(&parent, false);
        assert!(
            r.files
                .iter()
                .any(|(p, a)| *a == FileAction::Created && p.ends_with("SKILL.md"))
        );
        assert!(
            r.files
                .iter()
                .any(|(p, a)| *a == FileAction::Created && p.ends_with(SIDECAR_FILE_NAME))
        );
        // 内容逐字节等于嵌入。
        assert_eq!(fs::read_to_string(skill_file(&parent)).unwrap(), SKILL_MD);
        // sidecar 解析回来，hash == git_blob_sha1(SKILL_MD)。
        let (_, marker) = read_installed(&parent);
        let marker = marker.expect("sidecar written");
        assert_eq!(marker.hash, git_blob_sha1(SKILL_MD.as_bytes()));
        assert_eq!(marker.version, env!("CARGO_PKG_VERSION"));
        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn write_then_unchanged_no_op() {
        let parent = temp_parent("write-unchanged");
        write_skill_to_dir(&parent, false);
        let r = write_skill_to_dir(&parent, false);
        assert_eq!(r.files.len(), 1);
        assert_eq!(r.files[0].1, FileAction::Unchanged);
        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn write_skips_locally_modified_then_force_overwrites() {
        let parent = temp_parent("write-skip");
        write_skill_to_dir(&parent, false);
        fs::write(skill_file(&parent), "user hacked this\n").unwrap();

        let r = write_skill_to_dir(&parent, false);
        assert_eq!(r.files.len(), 1);
        assert_eq!(r.files[0].1, FileAction::Skipped);
        assert!(!r.notes.is_empty());
        assert_eq!(
            fs::read_to_string(skill_file(&parent)).unwrap(),
            "user hacked this\n"
        );

        let r2 = write_skill_to_dir(&parent, true);
        assert!(
            r2.files
                .iter()
                .any(|(p, a)| *a == FileAction::Updated && p.ends_with("SKILL.md"))
        );
        assert_eq!(fs::read_to_string(skill_file(&parent)).unwrap(), SKILL_MD);
        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn uninstall_removes_files_and_dir_then_status_not_installed() {
        let parent = temp_parent("uninstall");
        write_skill_to_dir(&parent, false);
        assert!(skill_file(&parent).exists());

        let r = uninstall_from_dir(&parent);
        let removed = r
            .files
            .iter()
            .filter(|(_, a)| *a == FileAction::Removed)
            .count();
        assert_eq!(removed, 2, "SKILL.md + sidecar removed");
        assert!(!skill_dir(&parent).exists(), "empty skill dir removed");
        assert_eq!(status_for_dir(&parent), SkillStatus::NotInstalled);
        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn uninstall_absent_is_not_found() {
        let parent = temp_parent("uninstall-absent");
        let r = uninstall_from_dir(&parent);
        assert_eq!(r.files.len(), 1);
        assert_eq!(r.files[0].1, FileAction::NotFound);
        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn status_reports_full_lifecycle() {
        let parent = temp_parent("status");
        assert_eq!(status_for_dir(&parent), SkillStatus::NotInstalled);

        write_skill_to_dir(&parent, false);
        assert_eq!(status_for_dir(&parent), SkillStatus::UpToDate);

        fs::write(skill_file(&parent), "edited\n").unwrap();
        assert_eq!(status_for_dir(&parent), SkillStatus::LocallyModified);

        let drifted = "old embedded\n";
        fs::write(skill_file(&parent), drifted).unwrap();
        let marker = SkillMarker {
            hash: git_blob_sha1(drifted.as_bytes()),
            version: "0.0.1".into(),
            installed_at: "t".into(),
        };
        fs::write(sidecar_file(&parent), marker.to_pretty_json()).unwrap();
        assert_eq!(status_for_dir(&parent), SkillStatus::Outdated);

        let _ = fs::remove_dir_all(&parent);
    }
}
