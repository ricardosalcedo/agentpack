use crate::manifest::{AgentDep, Manifest};
use anyhow::{bail, Result};

pub fn run(package: &str, is_agent: bool) -> Result<()> {
    let mut manifest = Manifest::load()?;

    let (name, version) = match package.rsplit_once('@') {
        Some((n, v)) => (n.to_string(), v.to_string()),
        None => (package.to_string(), "*".to_string()),
    };

    if version != "*" {
        semver::VersionReq::parse(&version)
            .map_err(|e| anyhow::anyhow!("Invalid version '{}': {}", version, e))?;
    }

    if is_agent {
        if manifest.agents.contains_key(&name) {
            bail!("'{}' is already an agent dependency.", name);
        }
        manifest
            .agents
            .insert(name.clone(), AgentDep::Version(version.clone()));
        manifest.save()?;
        println!("Added agent: {} @ {}", name, version);
    } else {
        if manifest.dependencies.contains_key(&name) {
            bail!("'{}' is already an MCP dependency.", name);
        }
        manifest.dependencies.insert(name.clone(), version.clone());
        manifest.save()?;
        println!("Added mcp: {} @ {}", name, version);
    }

    Ok(())
}
