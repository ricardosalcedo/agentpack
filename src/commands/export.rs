use anyhow::{bail, Result};
use serde_json::{json, Map};
use crate::manifest::LockFile;

pub fn run(target: &str) -> Result<()> {
    let lock = LockFile::load()?;

    match target {
        "claude-desktop" => export_claude(&lock),
        "vscode" => export_vscode(&lock),
        _ => bail!("Unknown target '{}'. Supported: claude-desktop, vscode", target),
    }
}

fn export_claude(lock: &LockFile) -> Result<()> {
    let mut servers = Map::new();

    for (name, entry) in &lock.resolved {
        let short_name = name.rsplit('/').next().unwrap_or(name);
        if let Some(t) = &entry.transport {
            let server = match t.transport_type.as_str() {
                "stdio" => json!({
                    "command": t.command.as_deref().unwrap_or("npx"),
                    "args": t.args,
                }),
                "streamable-http" | "sse" => json!({
                    "url": t.url.as_deref().unwrap_or(""),
                }),
                _ => json!({"command": "echo", "args": ["unsupported transport"]}),
            };
            servers.insert(short_name.to_string(), server);
        }
    }

    let config = json!({ "mcpServers": servers });
    let output = serde_json::to_string_pretty(&config)?;
    println!("{}", output);
    std::fs::write("claude_desktop_config.json", &output)?;
    println!("\n-> Written to claude_desktop_config.json");
    Ok(())
}

fn export_vscode(lock: &LockFile) -> Result<()> {
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

    let config = json!({ "mcp": { "servers": servers } });
    let output = serde_json::to_string_pretty(&config)?;
    println!("{}", output);
    std::fs::write(".vscode/mcp.json", &output)?;
    println!("\n-> Written to .vscode/mcp.json");
    Ok(())
}
