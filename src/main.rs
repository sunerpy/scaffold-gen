use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::process;

mod commands;
mod constants;
mod generators;
mod logging;
mod skill;
mod template_engine;
mod utils;

use commands::new::NewCommand;
use logging::Verbosity;

const FRAMEWORK_HELP: &str =
    "Framework type (gin, go-zero, mcp-server, tauri, vue3, react, fastapi, mcp-python, none)";
const LANGUAGE_HELP: &str = "Project language (go, rust, python, typescript)";

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(color = clap::ColorChoice::Auto)]
struct Cli {
    #[arg(
        short,
        long,
        global = true,
        help = "Suppress progress output (errors only)"
    )]
    quiet: bool,
    #[arg(short, long, global = true, help = "Enable verbose (debug) output")]
    verbose: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new project
    New {
        /// Project name
        name: String,
        /// Target directory (optional, defaults to current directory)
        #[arg(short, long)]
        path: Option<String>,
        /// Framework type
        #[arg(long, help = FRAMEWORK_HELP)]
        framework: Option<String>,
        /// Host address (for Go web frameworks)
        #[arg(long, help = "Host address (for Go web frameworks)")]
        host: Option<String>,
        /// HTTP port (for Go web frameworks)
        #[arg(long, help = "HTTP port (for Go web frameworks)")]
        port: Option<u16>,
        /// gRPC port (for go-zero framework)
        #[arg(long, help = "gRPC port (for go-zero framework)")]
        grpc_port: Option<u16>,
        /// Project language
        #[arg(long, help = LANGUAGE_HELP)]
        language: Option<String>,
        /// Enable pre-commit hooks
        #[arg(long, help = "Enable pre-commit hooks")]
        precommit: Option<bool>,
        /// License type (MIT, Apache-2.0, GPL-3.0, BSD-3-Clause, None)
        #[arg(
            long,
            help = "License type (MIT, Apache-2.0, GPL-3.0, BSD-3-Clause, None)"
        )]
        license: Option<String>,
        /// Enable Swagger documentation (for Gin framework)
        #[arg(long, help = "Enable Swagger documentation (for Gin framework)")]
        swagger: Option<bool>,
        /// Enable proto-gen tool (for Rust projects)
        #[arg(long, help = "Enable proto-gen tool (for Rust projects)")]
        proto_gen: Option<bool>,
        /// Enable error-gen tool (for Rust projects)
        #[arg(long, help = "Enable error-gen tool (for Rust projects)")]
        error_gen: Option<bool>,
        /// Generate a companion Makefile + Dockerfile (build/image automation)
        #[arg(
            long,
            help = "Generate a companion Makefile + Dockerfile (build/image automation)"
        )]
        with_build: Option<bool>,
        /// MCP Python backend (for mcp-python framework)
        #[arg(
            long,
            help = "MCP Python backend (fastmcp, official) — for mcp-python framework"
        )]
        mcp_backend: Option<String>,
        /// Optional auth mode (for mcp-python framework)
        #[arg(
            long,
            help = "Auth mode (none, jwt) — for mcp-python framework; default none"
        )]
        auth: Option<String>,
    },
    /// List available languages and frameworks
    List {
        /// Output machine-readable JSON instead of a table
        #[arg(long)]
        json: bool,
    },
    /// Print version, crate name, and repository URL
    Version,
    /// Generate shell completion scripts (bash, zsh, fish, powershell, elvish)
    Completions {
        /// Target shell
        shell: Shell,
        /// Install to the shell's completion location instead of printing to stdout
        #[arg(long)]
        install: bool,
    },
    /// Update the binary in place from the latest GitHub release
    SelfUpdate {
        /// Check for a newer release without installing it
        #[arg(long)]
        check: bool,
        /// Reinstall even if already on the latest version
        #[arg(long)]
        force: bool,
        /// Update to a specific version tag (e.g. v0.2.0) instead of latest
        #[arg(long)]
        tag: Option<String>,
    },
    /// Install the embedded scaffold-gen skill into AI agent skill directories
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
}

const SKILL_TARGET_HELP: &str = "Restrict to specific agents (opencode, claude, cursor, kiro); repeatable. Empty = all detected";

#[derive(Subcommand)]
enum SkillAction {
    /// Install the embedded skill into the selected agents' skill directories
    Install {
        /// Restrict to specific agents (repeatable)
        #[arg(long, help = SKILL_TARGET_HELP)]
        target: Vec<String>,
        /// Install into the global (user-level) skill directory (default)
        #[arg(long)]
        global: bool,
        /// Install into the local (project-level) skill directory
        #[arg(long, conflicts_with = "global")]
        local: bool,
        /// Proceed without confirmation (non-interactive; default behavior)
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Refresh installed skills to the embedded version
    Update {
        /// Restrict to specific agents (repeatable)
        #[arg(long, help = SKILL_TARGET_HELP)]
        target: Vec<String>,
        /// Operate on the global (user-level) skill directory (default)
        #[arg(long)]
        global: bool,
        /// Operate on the local (project-level) skill directory
        #[arg(long, conflicts_with = "global")]
        local: bool,
        /// Overwrite locally modified skill files
        #[arg(long)]
        force: bool,
    },
    /// Remove the installed skill from the selected agents
    Uninstall {
        /// Restrict to specific agents (repeatable)
        #[arg(long, help = SKILL_TARGET_HELP)]
        target: Vec<String>,
        /// Operate on the global (user-level) skill directory (default)
        #[arg(long)]
        global: bool,
        /// Operate on the local (project-level) skill directory
        #[arg(long, conflicts_with = "global")]
        local: bool,
        /// Proceed without confirmation (non-interactive; default behavior)
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Report per-agent skill installation status
    Status {
        /// Restrict to specific agents (repeatable)
        #[arg(long, help = SKILL_TARGET_HELP)]
        target: Vec<String>,
        /// Query the global (user-level) skill directory (default)
        #[arg(long)]
        global: bool,
        /// Query the local (project-level) skill directory
        #[arg(long, conflicts_with = "global")]
        local: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    logging::init(Verbosity::from_flags(cli.quiet, cli.verbose));

    let result = run(cli.command).await;

    if let Err(e) = result {
        tracing::error!("Error: {e}");
        process::exit(1);
    }
}

async fn run(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::New {
            name,
            path,
            framework,
            host,
            port,
            grpc_port,
            language,
            precommit,
            license,
            swagger,
            proto_gen,
            error_gen,
            with_build,
            mcp_backend,
            auth,
        } => {
            NewCommand::new(name, path)
                .with_framework(framework)
                .with_host(host)
                .with_port(port)
                .with_grpc_port(grpc_port)
                .with_language(language)
                .with_precommit(precommit)
                .with_license(license)
                .with_swagger(swagger)
                .with_proto_gen(proto_gen)
                .with_error_gen(error_gen)
                .with_build(with_build)
                .with_mcp_backend(mcp_backend)
                .with_auth_mode(auth)
                .execute()
                .await
        }
        Commands::List { json } => commands::list::execute(json),
        Commands::Version => {
            commands::version::execute();
            Ok(())
        }
        Commands::Completions { shell, install } => {
            let mut cmd = Cli::command();
            commands::completions::execute(shell, install, &mut cmd)
        }
        Commands::SelfUpdate { check, force, tag } => {
            commands::self_update::execute(check, force, tag)
        }
        Commands::Skill { action } => commands::skill::execute(skill_request_from(action)),
    }
}

fn skill_request_from(action: SkillAction) -> commands::skill::SkillRequest {
    use commands::skill::{Action, SkillRequest};
    match action {
        SkillAction::Install {
            target,
            global: _,
            local,
            yes: _,
        } => SkillRequest {
            action: Action::Install,
            targets: target,
            local,
            force: false,
        },
        SkillAction::Update {
            target,
            global: _,
            local,
            force,
        } => SkillRequest {
            action: Action::Update,
            targets: target,
            local,
            force,
        },
        SkillAction::Uninstall {
            target,
            global: _,
            local,
            yes: _,
        } => SkillRequest {
            action: Action::Uninstall,
            targets: target,
            local,
            force: false,
        },
        SkillAction::Status {
            target,
            global: _,
            local,
        } => SkillRequest {
            action: Action::Status,
            targets: target,
            local,
            force: false,
        },
    }
}

#[cfg(test)]
mod cli_tests {
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
        let cli =
            Cli::try_parse_from(["scafgen", "new", "x", "--with-build", "true"]).expect("parses");
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
        let cli =
            Cli::try_parse_from(["scafgen", "skill", "install"]).expect("skill install parses");
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
                action:
                    SkillAction::Install {
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
}
