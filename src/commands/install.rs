use anyhow::Result;
use crate::manifest::Manifest;
use crate::resolver;

pub fn run() -> Result<()> {
    let manifest = Manifest::load()?;

    if manifest.dependencies.is_empty() {
        println!("No dependencies to resolve.");
        return Ok(());
    }

    println!("Resolving {} dependencies...", manifest.dependencies.len());
    let lock = resolver::resolve(&manifest)?;

    lock.save()?;
    println!("Written agentpack.lock ({} packages resolved)", lock.resolved.len());
    Ok(())
}
