use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::process;

mod commands;
mod constants;
mod generators;
mod logging;
mod template_engine;
mod utils;

use commands::new::NewCommand;
use logging::Verbosity;

const FRAMEWORK_HELP: &str =
    "Framework type (gin, go-zero, mcp-server, tauri, vue3, react, fastapi, none)";
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
}
