use anyhow::Result;
use std::path::Path;

use crate::manifest::Manifest;

/// Search for MCP servers in the local catalog
pub fn run(query: &str) -> Result<()> {
    let query_lower = query.to_lowercase();
    let catalog_dir = find_catalog_dir();
    let mut results = vec![];

    if let Some(dir) = &catalog_dir {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let manifest_path = entry.path().join("agentpack.json");
            if !manifest_path.exists() {
                continue;
            }
            let content = std::fs::read_to_string(&manifest_path)?;
            if let Ok(m) = serde_json::from_str::<Manifest>(&content) {
                let matches = m.name.to_lowercase().contains(&query_lower)
                    || m.description.to_lowercase().contains(&query_lower)
                    || m.provides
                        .as_ref()
                        .map(|p| {
                            p.capabilities
                                .iter()
                                .any(|c| c.to_lowercase().contains(&query_lower))
                        })
                        .unwrap_or(false)
                    || m.tools
                        .iter()
                        .any(|t| t.name.to_lowercase().contains(&query_lower));

                if matches {
                    results.push(m);
                }
            }
        }
    }

    // Also search local packages/
    if Path::new("packages").exists() {
        for entry in std::fs::read_dir("packages")? {
            let entry = entry?;
            let manifest_path = entry.path().join("agentpack.json");
            if !manifest_path.exists() {
                continue;
            }
            let content = std::fs::read_to_string(&manifest_path)?;
            if let Ok(m) = serde_json::from_str::<Manifest>(&content) {
                let matches = m.name.to_lowercase().contains(&query_lower)
                    || m.description.to_lowercase().contains(&query_lower);
                if matches && !results.iter().any(|r: &Manifest| r.name == m.name) {
                    results.push(m);
                }
            }
        }
    }

    if results.is_empty() {
        println!("No servers found matching '{}'.", query);
        println!("Try: filesystem, search, database, browser, payments");
    } else {
        println!("Found {} servers matching '{}':\n", results.len(), query);
        for m in &results {
            let caps = m
                .provides
                .as_ref()
                .map(|p| p.capabilities.join(", "))
                .unwrap_or_default();
            let tools: Vec<_> = m.tools.iter().map(|t| t.name.as_str()).collect();
            println!("  {} @ {}", m.name, m.version);
            if !m.description.is_empty() {
                println!("    {}", m.description);
            }
            if !caps.is_empty() {
                println!("    capabilities: [{}]", caps);
            }
            if !tools.is_empty() {
                println!("    tools: [{}]", tools.join(", "));
            }
            println!();
        }
    }
    Ok(())
}

fn find_catalog_dir() -> Option<String> {
    // Check relative to binary or common locations
    let candidates = [
        "catalog/packages",
        "../catalog/packages",
        "../../catalog/packages",
    ];
    for c in candidates {
        if Path::new(c).exists() {
            return Some(c.to_string());
        }
    }
    // Check next to the executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let catalog = dir.join("../catalog/packages");
            if catalog.exists() {
                return Some(catalog.to_string_lossy().to_string());
            }
        }
    }
    None
}
