use anyhow::{bail, Result};
use serde_json::{json, Map, Value};
use crate::manifest::LockFile;

pub fn run(target: &str) -> Result<()> {
    let lock = LockFile::load()?;

    match target {
        "claude-desktop" => export_file(&lock, "mcpServers", "claude_desktop_config.json"),
        "vscode" | "copilot" => export_nested(&lock, "mcp", "servers", ".vscode/mcp.json"),
        "kiro" => export_nested(&lock, "mcp", "servers", ".kiro/mcp.json"),
        "cursor" => export_file(&lock, "mcpServers", ".cursor/mcp.json"),
        _ => bail!("Unknown target '{}'. Supported: claude-desktop, vscode, kiro, cursor", target),
    }
}

/// Format: { "<wrapper_key>": { "server-name": { ... } } }
fn export_file(lock: &LockFile, wrapper_key: &str, out_path: &str) -> Result<()> {
    let servers = build_servers(lock);
    let config = json!({ wrapper_key: servers });
    write_output(&config, out_path)
}

/// Format: { "<outer>": { "<inner>": { "server-name": { ... } } } }
fn export_nested(lock: &LockFile, outer: &str, inner: &str, out_path: &str) -> Result<()> {
    let servers = build_servers(lock);
    let config = json!({ outer: { inner: servers } });
    write_output(&config, out_path)
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
