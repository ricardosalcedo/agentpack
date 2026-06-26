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
        /// Add as agent dependency instead of MCP
        #[arg(long)]
        agent: bool,
    },
    /// Resolve dependencies and generate lock file
    Install,
    /// Display the dependency graph
    Graph,
    /// Export config for a target client
    Export {
        /// Target: claude-desktop, vscode
        #[arg(long)]
        target: String,
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => commands::init::run(),
        Commands::Add { package, agent } => commands::add::run(&package, agent),
        Commands::Install => commands::install::run(),
        Commands::Graph => commands::graph::run(),
        Commands::Export { target } => commands::export::run(&target),
        Commands::Run => commands::run::run(),
        Commands::Audit => commands::audit::run(),
        Commands::Fetch { name, source } => commands::fetch::run(&name, &source),
        Commands::Validate => commands::validate::run(),
    }
}
