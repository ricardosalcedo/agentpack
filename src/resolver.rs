use anyhow::Result;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

use crate::manifest::{AgentDep, LockFile, Manifest, ResolvedEntry, Source, Transport};

pub fn resolve(manifest: &Manifest, profile: Option<&str>) -> Result<LockFile> {
    let mut resolved = BTreeMap::new();

    let deps = manifest.deps_for_profile(profile);
    let agents = manifest.agents_for_profile(profile);

    for (name, version_req) in &deps {
        let entry = resolve_mcp(name, version_req)?;
        resolved.insert(name.clone(), entry);
    }

    for (name, agent_dep) in &agents {
        let entry = resolve_agent(name, agent_dep)?;
        resolved.insert(name.clone(), entry);
    }

    check_conflicts(&resolved)?;

    Ok(LockFile {
        lock_version: 1,
        resolved,
    })
}

fn resolve_mcp(name: &str, version_req: &str) -> Result<ResolvedEntry> {
    let local_path = format!("packages/{}/agentpack.json", name.replace('/', "__"));
    if Path::new(&local_path).exists() {
        let content = std::fs::read_to_string(&local_path)?;
        let m: Manifest = serde_json::from_str(&content)?;
        return Ok(ResolvedEntry {
            version: m.version.clone(),
            entry_type: "mcp-server".into(),
            source: Source {
                source_type: "local".into(),
                package: Some(local_path),
                version: Some(m.version),
            },
            integrity: Some(compute_integrity(&content)),
            transport: m.transport,
            dependencies: m.dependencies,
            agents: BTreeMap::new(),
            provides: m.provides,
        });
    }

    let resolved_version = resolve_version(version_req);
    let (source, transport) = infer_source(name, &resolved_version);
    let integrity = compute_integrity(&format!("{}@{}", name, resolved_version));

    Ok(ResolvedEntry {
        version: resolved_version,
        entry_type: "mcp-server".into(),
        source,
        integrity: Some(integrity),
        transport: Some(transport),
        dependencies: BTreeMap::new(),
        agents: BTreeMap::new(),
        provides: None,
    })
}

fn resolve_agent(name: &str, dep: &AgentDep) -> Result<ResolvedEntry> {
    let local_path = format!("packages/{}/agentpack.json", name.replace('/', "__"));
    if Path::new(&local_path).exists() {
        let content = std::fs::read_to_string(&local_path)?;
        let m: Manifest = serde_json::from_str(&content)?;
        let agent_deps: BTreeMap<String, String> = m
            .agents
            .iter()
            .map(|(k, v)| (k.clone(), v.version().to_string()))
            .collect();
        return Ok(ResolvedEntry {
            version: m.version.clone(),
            entry_type: "agent".into(),
            source: Source {
                source_type: "local".into(),
                package: Some(local_path),
                version: Some(m.version),
            },
            integrity: Some(compute_integrity(&content)),
            transport: m.transport,
            dependencies: m.dependencies,
            agents: agent_deps,
            provides: m.provides,
        });
    }

    let source_url = dep.source().unwrap_or("registry");
    let resolved_version = resolve_version(dep.version());
    let integrity = compute_integrity(&format!("{}@{}", name, resolved_version));

    Ok(ResolvedEntry {
        version: resolved_version,
        entry_type: "agent".into(),
        source: Source {
            source_type: source_url.into(),
            package: Some(name.into()),
            version: Some(dep.version().into()),
        },
        integrity: Some(integrity),
        transport: None,
        dependencies: BTreeMap::new(),
        agents: BTreeMap::new(),
        provides: None,
    })
}

pub fn compute_integrity(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("sha256-{}", hex::encode(hasher.finalize()))
}

fn resolve_version(req: &str) -> String {
    if req == "*" {
        return "latest".into();
    }
    let first = req.split(',').next().unwrap_or(req).trim();
    first
        .trim_start_matches('^')
        .trim_start_matches('~')
        .trim_start_matches(">=")
        .trim_start_matches('>')
        .trim_start_matches("<=")
        .trim_start_matches('<')
        .trim_start_matches('=')
        .trim()
        .to_string()
}

fn infer_source(name: &str, version: &str) -> (Source, Transport) {
    let package_name = name
        .strip_prefix("io.github.")
        .unwrap_or(name)
        .replace('/', "-");
    let npx_package = if version == "latest" {
        format!("@mcp/{}", package_name)
    } else {
        format!("@mcp/{}@{}", package_name, version)
    };
    (
        Source {
            source_type: "npm".into(),
            package: Some(npx_package.clone()),
            version: Some(version.into()),
        },
        Transport {
            transport_type: "stdio".into(),
            command: Some("npx".into()),
            args: vec!["-y".into(), npx_package],
            url: None,
        },
    )
}

fn check_conflicts(resolved: &BTreeMap<String, ResolvedEntry>) -> Result<()> {
    let mut tool_owners: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for name in resolved.keys() {
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

    let conflicts: Vec<_> = tool_owners.iter().filter(|(_, o)| o.len() > 1).collect();
    if !conflicts.is_empty() {
        eprintln!("⚠ Tool name conflicts detected:");
        for (tool, owners) in &conflicts {
            eprintln!("  '{}' provided by: {}", tool, owners.join(", "));
        }
        eprintln!("  Consider using toolNamespace prefixing.");
    }
    Ok(())
}
