use anyhow::Result;

use crate::manifest::Manifest;
use crate::resolver;

pub fn run(profile: Option<&str>) -> Result<()> {
    let mut manifest = Manifest::load()?;

    // Resolve semantic capability requirements
    if !manifest.requires.is_empty() {
        println!(
            "Resolving {} capability requirements...",
            manifest.requires.len()
        );
        manifest.resolve_capabilities()?;
        manifest.save()?;
        for req in &manifest.requires {
            if let Some(resolved) = &req.resolved_by {
                println!("  '{}' → {}", req.capability, resolved);
            } else {
                eprintln!("  ⚠ '{}' — no provider found", req.capability);
            }
        }
    }

    let lock = resolver::resolve(&manifest, profile)?;

    if lock.resolved.is_empty() {
        println!("No dependencies to resolve.");
        return Ok(());
    }

    lock.save()?;
    let suffix = profile
        .map(|p| format!(" (profile: {})", p))
        .unwrap_or_default();
    println!(
        "Written agentpack.lock ({} packages resolved{})",
        lock.resolved.len(),
        suffix
    );
    Ok(())
}
