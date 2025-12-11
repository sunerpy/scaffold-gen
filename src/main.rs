use clap::{Parser, Subcommand};
use colored::*;
use std::process;

mod commands;
mod constants;
mod generators;
mod scaffold;
mod template_engine;
mod utils;

use commands::new::NewCommand;

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(color = clap::ColorChoice::Auto)]
struct Cli {
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
        /// Framework type (gin or go-zero)
        #[arg(long, help = "Framework type (gin or go-zero)")]
        framework: Option<String>,
        /// Host address
        #[arg(long)]
        host: Option<String>,
        /// HTTP port
        #[arg(long)]
        port: Option<u16>,
        /// gRPC port
        #[arg(long)]
        grpc_port: Option<u16>,
        /// Project language (go, etc.)
        #[arg(long, help = "Project language (go, etc.)")]
        language: Option<String>,
        /// Enable pre-commit hooks
        #[arg(long)]
        precommit: Option<bool>,
        /// License type
        #[arg(long)]
        license: Option<String>,
        /// Enable Swagger documentation
        #[arg(long)]
        swagger: Option<bool>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
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
        } => {
            let new_cmd = NewCommand::new(name, path)
                .with_framework(framework)
                .with_host(host)
                .with_port(port)
                .with_grpc_port(grpc_port)
                .with_language(language)
                .with_precommit(precommit)
                .with_license(license)
                .with_swagger(swagger);
            new_cmd.execute().await
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}
