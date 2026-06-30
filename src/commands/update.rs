use anyhow::Result;
use semver::Version;
use std::path::Path;

use crate::manifest::{LockFile, Manifest};
use crate::resolver;

pub fn run() -> Result<()> {
    let manifest = Manifest::load()?;
    let mut updated: Vec<String> = Vec::new();
    let mut registry_checks: Vec<String> = Vec::new();

    // Load existing lock file if it exists
    let existing_lock = LockFile::load().ok();

    for (name, version_req) in &manifest.dependencies {
        let local_path = format!("packages/{}/agentpack.json", name.replace('/', "__"));
        if Path::new(&local_path).exists() {
            // Local package: check if local version is newer than what's in lock
            let content = std::fs::read_to_string(&local_path)?;
            let pkg: Manifest = serde_json::from_str(&content)?;

            if let Some(ref lock) = existing_lock {
                if let Some(entry) = lock.resolved.get(name) {
                    if let (Ok(locked_ver), Ok(local_ver)) =
                        (Version::parse(&entry.version), Version::parse(&pkg.version))
                    {
                        if local_ver > locked_ver {
                            updated.push(format!(
                                "{}: {} → {} (local)",
                                name, entry.version, pkg.version
                            ));
                        }
                    }
                } else {
                    // New dep not yet in lock
                    updated.push(format!("{}: (new) → {} (local)", name, pkg.version));
                }
            } else {
                // No lock file exists, everything is new
                updated.push(format!("{}: (new) → {} (local)", name, pkg.version));
            }
        } else {
            // Inferred/registry package
            registry_checks.push(format!(
                "{} ({}): check registry for updates",
                name, version_req
            ));
        }
    }

    for (name, agent_dep) in &manifest.agents {
        let local_path = format!("packages/{}/agentpack.json", name.replace('/', "__"));
        if Path::new(&local_path).exists() {
            let content = std::fs::read_to_string(&local_path)?;
            let pkg: Manifest = serde_json::from_str(&content)?;

            if let Some(ref lock) = existing_lock {
                if let Some(entry) = lock.resolved.get(name) {
                    if let (Ok(locked_ver), Ok(local_ver)) =
                        (Version::parse(&entry.version), Version::parse(&pkg.version))
                    {
                        if local_ver > locked_ver {
                            updated.push(format!(
                                "{}: {} → {} (local agent)",
                                name, entry.version, pkg.version
                            ));
                        }
                    }
                } else {
                    updated.push(format!("{}: (new) → {} (local agent)", name, pkg.version));
                }
            } else {
                updated.push(format!("{}: (new) → {} (local agent)", name, pkg.version));
            }
        } else {
            registry_checks.push(format!(
                "{} ({}): check registry for updates",
                name,
                agent_dep.version()
            ));
        }
    }

    // Re-resolve and write the lock file if there are local updates
    if !updated.is_empty() {
        let lock = resolver::resolve(&manifest, None)?;
        lock.save()?;
        println!("Updated agentpack.lock:");
        for u in &updated {
            println!("  ✓ {}", u);
        }
    } else {
        println!("All local packages are up to date.");
    }

    if !registry_checks.is_empty() {
        println!("\nRegistry packages:");
        for r in &registry_checks {
            println!("  ℹ {}", r);
        }
    }

    Ok(())
}
