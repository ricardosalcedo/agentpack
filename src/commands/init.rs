use anyhow::{bail, Result};
use std::collections::BTreeMap;
use std::env;

use crate::manifest::Manifest;

pub fn run() -> Result<()> {
    if Manifest::exists() {
        bail!("agentpack.json already exists in this directory.");
    }

    let dir_name = env::current_dir()?
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "my-project".into());

    let manifest = Manifest {
        name: format!("io.github.user/{}", dir_name),
        version: "0.1.0".into(),
        description: String::new(),
        pkg_type: "composite".into(),
        transport: None,
        dependencies: BTreeMap::new(),
        agents: BTreeMap::new(),
        provides: None,
        requires: vec![],
        profiles: BTreeMap::new(),
        tools: vec![],
    };

    manifest.save()?;
    println!("Created agentpack.json");
    Ok(())
}
