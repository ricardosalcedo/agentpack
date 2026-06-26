use anyhow::{bail, Result};
use serde_json::{json, Map, Value};

use crate::manifest::{LockFile, Manifest};

pub fn run(target: &str, profile: Option<&str>) -> Result<()> {
    let lock = LockFile::load()?;

    let filtered = match profile {
        Some(p) => {
            let manifest = Manifest::load()?;
            if !manifest.profiles.contains_key(p) {
                bail!(
                    "Profile '{}' not found. Available: {}",
                    p,
                    manifest
                        .profiles
                        .keys()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            lock.filter_by_profile(&manifest, p)
        }
        None => lock,
    };

    match target {
        "claude-desktop" => export_file(&filtered, "mcpServers", "claude_desktop_config.json"),
        "vscode" | "copilot" => export_nested(&filtered, "mcp", "servers", ".vscode/mcp.json"),
        "kiro" => export_nested(&filtered, "mcp", "servers", ".kiro/mcp.json"),
        "cursor" => export_file(&filtered, "mcpServers", ".cursor/mcp.json"),
        "gateway" => export_gateway(&filtered),
        _ => bail!(
            "Unknown target '{}'. Supported: claude-desktop, vscode, kiro, cursor, gateway",
            target
        ),
    }
}

fn export_file(lock: &LockFile, wrapper_key: &str, out_path: &str) -> Result<()> {
    let servers = build_servers(lock);
    let config = json!({ wrapper_key: servers });
    write_output(&config, out_path)
}

fn export_nested(lock: &LockFile, outer: &str, inner: &str, out_path: &str) -> Result<()> {
    let servers = build_servers(lock);
    let config = json!({ outer: { inner: servers } });
    write_output(&config, out_path)
}

fn export_gateway(_lock: &LockFile) -> Result<()> {
    // For gateway mode, export a config that points to agentpack-gateway as the single server
    let config = json!({
        "mcpServers": {
            "agentpack": {
                "command": "node",
                "args": ["node_modules/@agentpack/gateway/index.js", "--lock", "./agentpack.lock"],
                "type": "stdio"
            }
        }
    });
    write_output(&config, "agentpack-gateway.json")
}

fn build_servers(lock: &LockFile) -> Map<String, Value> {
    let mut servers = Map::new();
    for (name, entry) in &lock.resolved {
        let short_name = name.rsplit('/').next().unwrap_or(name);
        if let Some(t) = &entry.transport {
            let server = match t.transport_type.as_str() {
                "stdio" => json!({
                    "type": "stdio",
                    "command": t.command.as_deref().unwrap_or("npx"),
                    "args": t.args,
                }),
                "streamable-http" | "sse" => json!({
                    "type": "sse",
                    "url": t.url.as_deref().unwrap_or(""),
                }),
                _ => json!({"type": "stdio", "command": "echo", "args": ["unsupported"]}),
            };
            servers.insert(short_name.to_string(), server);
        }
    }
    servers
}

fn write_output(config: &Value, path: &str) -> Result<()> {
    let output = serde_json::to_string_pretty(config)?;
    println!("{}", output);
    if let Some(dir) = std::path::Path::new(path).parent() {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir)?;
        }
    }
    std::fs::write(path, &output)?;
    println!("\n-> Written to {}", path);
    Ok(())
}
