use anyhow::{Context, Result};
use std::process::Command;

use crate::manifest::Manifest;

/// Fetch a manifest from a remote source.
/// Supports:
/// - GitHub raw URLs: github:owner/repo/path
/// - Direct URLs: https://...
/// - Registry API (future): registry:package-name
pub fn fetch_manifest(source: &str) -> Result<Manifest> {
    let url = resolve_url(source)?;
    let body = http_get(&url)?;
    let manifest: Manifest = serde_json::from_str(&body)
        .with_context(|| format!("Failed to parse manifest from {}", url))?;
    Ok(manifest)
}

fn resolve_url(source: &str) -> Result<String> {
    if source.starts_with("https://") || source.starts_with("http://") {
        return Ok(source.to_string());
    }
    if let Some(path) = source.strip_prefix("file://") {
        return Ok(format!("file://{}", path));
    }
    if let Some(rest) = source.strip_prefix("github:") {
        // Format: github:owner/repo/path/to/agentpack.json
        // or github:owner/repo (defaults to agentpack.json at root)
        let parts: Vec<&str> = rest.splitn(3, '/').collect();
        let (owner, repo, path) = match parts.len() {
            2 => (parts[0], parts[1], "agentpack.json"),
            3 => (parts[0], parts[1], parts[2]),
            _ => anyhow::bail!("Invalid github source: {}", source),
        };
        return Ok(format!(
            "https://raw.githubusercontent.com/{}/{}/main/{}",
            owner, repo, path
        ));
    }
    anyhow::bail!(
        "Unknown source format: '{}'. Use https:// or github:owner/repo",
        source
    )
}

/// Minimal HTTP GET using curl (avoids adding reqwest as a dependency)
fn http_get(url: &str) -> Result<String> {
    let output = Command::new("curl")
        .args(["-sfL", "--max-time", "10", url])
        .output()
        .context("Failed to execute curl. Is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("HTTP request failed for {}: {}", url, stderr.trim());
    }

    String::from_utf8(output.stdout).context("Response was not valid UTF-8")
}

/// Fetch and save a remote manifest to the local packages/ directory
pub fn fetch_and_cache(name: &str, source: &str) -> Result<Manifest> {
    let manifest = fetch_manifest(source)?;
    let dir = format!("packages/{}", name.replace('/', "__"));
    std::fs::create_dir_all(&dir)?;
    let path = format!("{}/agentpack.json", dir);
    let json = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(&path, &json)?;
    println!("  Cached {} -> {}", name, path);
    Ok(manifest)
}
