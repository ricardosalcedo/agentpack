use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;

use crate::manifest::{LockFile, Manifest};
use crate::resolver::compute_integrity;

pub fn run() -> Result<()> {
    let manifest = Manifest::load()?;
    let mut issues: Vec<String> = vec![];

    // 1. Check for unpinned versions
    for (name, ver) in &manifest.dependencies {
        if ver == "*" || ver == "latest" {
            issues.push(format!(
                "[CRITICAL] Unpinned version for '{}': '{}'",
                name, ver
            ));
        }
    }

    // 2. Check lock file exists and integrity hashes are present
    let lock_exists = Path::new("agentpack.lock").exists();
    if !lock_exists {
        issues.push("[HIGH] No agentpack.lock found. Run `agentpack install`.".into());
    } else {
        let lock = LockFile::load()?;

        // 3. Verify integrity hashes
        for (name, entry) in &lock.resolved {
            match &entry.integrity {
                None => {
                    issues.push(format!("[HIGH] Missing integrity hash for '{}'", name));
                }
                Some(hash) => {
                    // For local packages, verify the hash still matches
                    let local_path = format!("packages/{}/agentpack.json", name.replace('/', "__"));
                    if Path::new(&local_path).exists() {
                        let content = std::fs::read_to_string(&local_path)?;
                        let current = compute_integrity(&content);
                        if &current != hash {
                            issues.push(format!(
                                "[CRITICAL] Integrity mismatch for '{}': lock says {} but file hashes to {}",
                                name, hash, current
                            ));
                        }
                    }
                }
            }
        }

        // 4. Check for tool name conflicts
        let mut tool_owners: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for name in lock.resolved.keys() {
            let local_path = format!("packages/{}/agentpack.json", name.replace('/', "__"));
            if Path::new(&local_path).exists() {
                if let Ok(content) = std::fs::read_to_string(&local_path) {
                    if let Ok(m) = serde_json::from_str::<Manifest>(&content) {
                        for tool in &m.tools {
                            tool_owners
                                .entry(tool.name.clone())
                                .or_default()
                                .push(name.clone());
                        }
                    }
                }
            }
        }
        for (tool, owners) in &tool_owners {
            if owners.len() > 1 {
                issues.push(format!(
                    "[WARN] Tool name conflict: '{}' provided by: {}",
                    tool,
                    owners.join(", ")
                ));
            }
        }

        // 5. Check for npx without version pinning in transport args
        for (name, entry) in &lock.resolved {
            if let Some(t) = &entry.transport {
                if t.command.as_deref() == Some("npx") {
                    let has_version = t.args.iter().any(|a| a.contains('@'));
                    if !has_version {
                        issues.push(format!(
                            "[HIGH] Unpinned npx in '{}': no version in args",
                            name
                        ));
                    }
                }
            }
        }

        // 6. Check credentials file exists if dependencies likely need secrets
        if !Path::new("agentpack.credentials.yaml").exists() && !lock.resolved.is_empty() {
            issues.push("[INFO] No agentpack.credentials.yaml found. Credentials will rely on ambient env vars.".into());
        }
    }

    // Report
    if issues.is_empty() {
        println!(
            "✓ No issues found. {} dependencies audited.",
            manifest.dependencies.len()
        );
    } else {
        println!("Audit results ({} issues):\n", issues.len());
        for issue in &issues {
            println!("  {}", issue);
        }
        println!();
        let critical = issues.iter().filter(|i| i.contains("[CRITICAL]")).count();
        let high = issues.iter().filter(|i| i.contains("[HIGH]")).count();
        let warn = issues.iter().filter(|i| i.contains("[WARN]")).count();
        println!(
            "Summary: {} critical, {} high, {} warnings",
            critical, high, warn
        );
    }

    Ok(())
}
