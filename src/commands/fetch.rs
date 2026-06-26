use crate::registry;
use anyhow::Result;

/// Fetch a remote manifest and cache it locally.
/// Usage: agentpack fetch io.github.anthropic/filesystem --source github:anthropic/mcp-filesystem
pub fn run(name: &str, source: &str) -> Result<()> {
    println!("Fetching manifest for '{}'...", name);
    let manifest = registry::fetch_and_cache(name, source)?;
    println!(
        "  ✓ {} @ {} ({} tools)",
        manifest.name,
        manifest.version,
        manifest.tools.len()
    );
    Ok(())
}
