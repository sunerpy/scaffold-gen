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
            help = "Auth mode (none, jwt, azure-ad) — for mcp-python framework; default none"
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
#[path = "main_tests.rs"]
mod cli_tests;
