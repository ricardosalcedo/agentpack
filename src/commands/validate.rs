use anyhow::Result;
use std::path::Path;

use crate::manifest::{LockFile, Manifest};

/// Validate that required capabilities from agent deps are satisfied by
/// what those agents declare in their `provides` block.
pub fn run() -> Result<()> {
    let manifest = Manifest::load()?;
    let mut issues: Vec<String> = vec![];

    if manifest.agents.is_empty() {
        println!("✓ No agent dependencies to validate.");
        return Ok(());
    }

    // Check each agent dep's required capabilities against what the agent provides
    for (name, dep) in &manifest.agents {
        let required = dep.required_capabilities();
        if required.is_empty() {
            continue;
        }

        // Load the agent's manifest to check its provides block
        let local_path = format!("packages/{}/agentpack.json", name.replace('/', "__"));
        if !Path::new(&local_path).exists() {
            issues.push(format!(
                "[WARN] Cannot validate '{}': no local manifest. Run `agentpack fetch` first.", name
            ));
            continue;
        }

        let content = std::fs::read_to_string(&local_path)?;
        let agent_manifest: Manifest = serde_json::from_str(&content)?;

        let provided: Vec<&str> = agent_manifest.provides
            .as_ref()
            .map(|p| p.capabilities.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default();

        for cap in required {
            if !provided.contains(&cap.as_str()) {
                issues.push(format!(
                    "[ERROR] '{}' requires capability '{}' but agent does not provide it. Available: [{}]",
                    name, cap, provided.join(", ")
                ));
            }
        }
    }

    // Also validate that agent deps have their own MCP deps resolved
    let lock_exists = Path::new("agentpack.lock").exists();
    if lock_exists {
        let lock = LockFile::load()?;
        for (name, entry) in &lock.resolved {
            if entry.entry_type == "agent" {
                for dep in entry.dependencies.keys() {
                    if !lock.resolved.contains_key(dep) {
                        issues.push(format!(
                            "[ERROR] Agent '{}' needs MCP '{}' which is not in the resolved graph.",
                            name, dep
                        ));
                    }
                }
                for dep in entry.agents.keys() {
                    if !lock.resolved.contains_key(dep) {
                        issues.push(format!(
                            "[ERROR] Agent '{}' needs agent '{}' which is not in the resolved graph.",
                            name, dep
                        ));
                    }
                }
            }
        }
    }

    if issues.is_empty() {
        println!("✓ All agent capabilities validated. {} agent deps checked.", manifest.agents.len());
    } else {
        println!("Validation issues ({}):\n", issues.len());
        for issue in &issues {
            println!("  {}", issue);
        }
    }

    Ok(())
}
