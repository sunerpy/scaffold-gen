use super::*;

#[test]
fn cli_command_builds_without_panic() {
    // clap 在 debug 下会对命令定义做断言校验；构建即验证整棵命令树合法。
    Cli::command().debug_assert();
}

#[test]
fn new_subcommand_still_parses() {
    let cli = Cli::try_parse_from(["scafgen", "new", "myproj", "--framework", "gin"])
        .expect("new parses");
    match cli.command {
        Commands::New {
            name, framework, ..
        } => {
            assert_eq!(name, "myproj");
            assert_eq!(framework.as_deref(), Some("gin"));
        }
        _ => panic!("expected New variant"),
    }
}

#[test]
fn list_parses_with_and_without_json() {
    let plain = Cli::try_parse_from(["scafgen", "list"]).expect("list parses");
    assert!(matches!(plain.command, Commands::List { json: false }));

    let json = Cli::try_parse_from(["scafgen", "list", "--json"]).expect("list --json parses");
    assert!(matches!(json.command, Commands::List { json: true }));
}

#[test]
fn version_parses() {
    let cli = Cli::try_parse_from(["scafgen", "version"]).expect("version parses");
    assert!(matches!(cli.command, Commands::Version));
}

#[test]
fn completions_parses_for_each_shell() {
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let cli = Cli::try_parse_from(["scafgen", "completions", shell])
            .unwrap_or_else(|e| panic!("completions {shell} parses: {e}"));
        assert!(matches!(
            cli.command,
            Commands::Completions { install: false, .. }
        ));
    }

    let installed = Cli::try_parse_from(["scafgen", "completions", "bash", "--install"])
        .expect("completions --install parses");
    assert!(matches!(
        installed.command,
        Commands::Completions { install: true, .. }
    ));
}

#[test]
fn self_update_parses_all_flags() {
    let check = Cli::try_parse_from(["scafgen", "self-update", "--check"])
        .expect("self-update --check parses");
    assert!(matches!(
        check.command,
        Commands::SelfUpdate {
            check: true,
            force: false,
            tag: None
        }
    ));

    let force = Cli::try_parse_from(["scafgen", "self-update", "--force"])
        .expect("self-update --force parses");
    assert!(matches!(
        force.command,
        Commands::SelfUpdate { force: true, .. }
    ));

    let tagged = Cli::try_parse_from(["scafgen", "self-update", "--tag", "v0.2.0"])
        .expect("self-update --tag parses");
    match tagged.command {
        Commands::SelfUpdate { tag, .. } => assert_eq!(tag.as_deref(), Some("v0.2.0")),
        _ => panic!("expected SelfUpdate variant"),
    }
}

#[test]
fn global_quiet_verbose_work_on_subcommands() {
    let cli = Cli::try_parse_from(["scafgen", "-q", "list"]).expect("global -q parses");
    assert!(cli.quiet);
    let cli = Cli::try_parse_from(["scafgen", "-v", "version"]).expect("global -v parses");
    assert!(cli.verbose);
}

#[test]
fn new_parses_with_build_flag() {
    let cli = Cli::try_parse_from(["scafgen", "new", "x", "--with-build", "true"]).expect("parses");
    match cli.command {
        Commands::New { with_build, .. } => assert_eq!(with_build, Some(true)),
        _ => panic!("expected New variant"),
    }

    let without = Cli::try_parse_from(["scafgen", "new", "x"]).expect("parses without flag");
    match without.command {
        Commands::New { with_build, .. } => assert_eq!(with_build, None),
        _ => panic!("expected New variant"),
    }
}

#[test]
fn framework_help_lists_all_current_frameworks() {
    for fw in [
        "gin",
        "go-zero",
        "mcp-server",
        "tauri",
        "vue3",
        "react",
        "fastapi",
        "none",
    ] {
        assert!(
            FRAMEWORK_HELP.contains(fw),
            "framework help missing {fw}: {FRAMEWORK_HELP}"
        );
    }
}

#[test]
fn skill_install_parses_defaults() {
    let cli = Cli::try_parse_from(["scafgen", "skill", "install"]).expect("skill install parses");
    match cli.command {
        Commands::Skill {
            action:
                SkillAction::Install {
                    target,
                    global,
                    local,
                    yes,
                },
        } => {
            assert!(target.is_empty());
            assert!(!global);
            assert!(!local);
            assert!(!yes);
        }
        _ => panic!("expected Skill::Install variant"),
    }
}

#[test]
fn skill_install_parses_target_and_local() {
    let cli = Cli::try_parse_from([
        "scafgen", "skill", "install", "--target", "opencode", "--target", "claude", "--local",
        "-y",
    ])
    .expect("skill install --target --local parses");
    match cli.command {
        Commands::Skill {
            action: SkillAction::Install {
                target, local, yes, ..
            },
        } => {
            assert_eq!(target, vec!["opencode".to_string(), "claude".to_string()]);
            assert!(local);
            assert!(yes);
        }
        _ => panic!("expected Skill::Install variant"),
    }
}

#[test]
fn skill_update_parses_force() {
    let cli = Cli::try_parse_from(["scafgen", "skill", "update", "--force"])
        .expect("skill update --force parses");
    match cli.command {
        Commands::Skill {
            action: SkillAction::Update { force, .. },
        } => assert!(force),
        _ => panic!("expected Skill::Update variant"),
    }
}

#[test]
fn skill_uninstall_parses_yes() {
    let cli = Cli::try_parse_from(["scafgen", "skill", "uninstall", "-y"])
        .expect("skill uninstall -y parses");
    match cli.command {
        Commands::Skill {
            action: SkillAction::Uninstall { yes, .. },
        } => assert!(yes),
        _ => panic!("expected Skill::Uninstall variant"),
    }
}

#[test]
fn skill_status_parses_global() {
    let cli = Cli::try_parse_from(["scafgen", "skill", "status", "--global"])
        .expect("skill status --global parses");
    match cli.command {
        Commands::Skill {
            action: SkillAction::Status { global, local, .. },
        } => {
            assert!(global);
            assert!(!local);
        }
        _ => panic!("expected Skill::Status variant"),
    }
}

#[test]
fn skill_global_and_local_conflict() {
    let err = Cli::try_parse_from(["scafgen", "skill", "status", "--global", "--local"]);
    assert!(err.is_err(), "--global and --local must conflict");
}
