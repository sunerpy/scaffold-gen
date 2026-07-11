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
fn run_with_cwd_set_still_maps_missing_binary_to_error() {
    // current_dir 分支被走到（cwd = 一个真实存在的临时目录），但可执行文件不存在 -> Err（不 panic）。
    let dir = std::env::temp_dir();
    let result = ExternalCommand::new("definitely-not-a-real-tool-xyz")
        .arg("version")
        .current_dir(&dir)
        .run();
    assert!(result.is_err());
}

#[test]
fn tool_available_true_for_present_binary() {
    // 单元测试环境保证有 cargo（由它运行），因此 which 应解析成功。
    assert!(tool_available("cargo"));
}

#[test]
fn command_outcome_accessors_expose_fields() {
    let outcome = CommandOutcome {
        success: true,
        stdout: "out".to_string(),
        stderr: "err".to_string(),
    };
    assert!(outcome.success());
    assert_eq!(outcome.stdout(), "out");
    assert_eq!(outcome.stderr(), "err");
}

#[test]
fn builder_records_cwd_and_args() {
    let cmd = ExternalCommand::new("go")
        .args(["mod", "tidy"])
        .current_dir("/some/dir");
    assert_eq!(cmd.args, ["mod", "tidy"]);
    assert_eq!(cmd.cwd, Some(PathBuf::from("/some/dir")));
}

#[test]
fn run_captures_output_of_a_real_command() {
    // cargo 必然存在（正在运行本测试）；走完整 output() 成功路径并捕获 stdout。
    let outcome = ExternalCommand::new("cargo")
        .arg("--version")
        .run()
        .expect("cargo --version runs");
    assert!(outcome.success(), "cargo --version exits 0");
    assert!(
        outcome.stdout().contains("cargo"),
        "stdout captured: {}",
        outcome.stdout()
    );
}
