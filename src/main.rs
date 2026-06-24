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
        /// Framework type (gin, go-zero, tauri, vue3, react, none)
        #[arg(long, help = "Framework type (gin, go-zero, tauri, vue3, react, none)")]
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
        /// Project language (go, rust, python, typescript)
        #[arg(long, help = "Project language (go, rust, python, typescript)")]
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
            proto_gen,
            error_gen,
        } => {
            let new_cmd = NewCommand::new(name, path)
                .with_framework(framework)
                .with_host(host)
                .with_port(port)
                .with_grpc_port(grpc_port)
                .with_language(language)
                .with_precommit(precommit)
                .with_license(license)
                .with_swagger(swagger)
                .with_proto_gen(proto_gen)
                .with_error_gen(error_gen);
            new_cmd.execute().await
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}
