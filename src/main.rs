mod commands;
mod manifest;
mod registry;
mod resolver;

#[cfg(test)]
mod tests;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "agentpack",
    version,
    about = "Dependency manager for MCP servers and AI agents"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new agentpack.json
    Init,
    /// Add a dependency
    Add {
        /// Package (e.g. io.github.anthropic/filesystem@^1.2.0)
        package: String,
        /// Add as agent dependency
        #[arg(long)]
        agent: bool,
    },
    /// Resolve dependencies and generate lock file
    Install {
        /// Only resolve deps in this profile
        #[arg(long)]
        profile: Option<String>,
    },
    /// Display the dependency graph
    Graph,
    /// Export config for a target client
    Export {
        /// Target: claude-desktop, vscode, kiro, cursor, gateway
        #[arg(long)]
        target: String,
        /// Export only deps in this profile
        #[arg(long)]
        profile: Option<String>,
    },
    /// Start all dependency servers/agents in order
    Run,
    /// Check for security and configuration issues
    Audit,
    /// Fetch a remote manifest and cache locally
    Fetch {
        /// Package name
        name: String,
        /// Source URL (https://, github:owner/repo, or file://)
        #[arg(long)]
        source: String,
    },
    /// Validate agent capability requirements
    Validate,
    /// Import an existing MCP server by introspecting its tools
    Import {
        /// Package name (e.g. io.github.stripe/payments)
        #[arg(long)]
        name: String,
        /// Command to start the server (followed by its args after --)
        #[arg(long)]
        command: String,
        /// Arguments for the command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => commands::init::run(),
        Commands::Add { package, agent } => commands::add::run(&package, agent),
        Commands::Install { profile } => commands::install::run(profile.as_deref()),
        Commands::Graph => commands::graph::run(),
        Commands::Export { target, profile } => commands::export::run(&target, profile.as_deref()),
        Commands::Run => commands::run::run(),
        Commands::Audit => commands::audit::run(),
        Commands::Fetch { name, source } => commands::fetch::run(&name, &source),
        Commands::Validate => commands::validate::run(),
        Commands::Import {
            name,
            command,
            args,
        } => commands::import::run(&name, &command, &args),
    }
}
