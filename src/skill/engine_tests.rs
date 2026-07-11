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
