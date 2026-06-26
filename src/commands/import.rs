use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::process::Command;

use crate::manifest::{Manifest, Tool, Transport};

/// Import an existing MCP server by introspecting its tools via a helper script.
pub fn run(name: &str, cmd: &str, args: &[String]) -> Result<()> {
    println!("Introspecting MCP server: {} {}", cmd, args.join(" "));

    // Use Node.js MCP SDK client to connect and list tools
    let script = format!(
        r#"
import {{ Client }} from "@modelcontextprotocol/sdk/client/index.js";
import {{ StdioClientTransport }} from "@modelcontextprotocol/sdk/client/stdio.js";
const transport = new StdioClientTransport({{ command: "{cmd}", args: {args_json} }});
const client = new Client({{ name: "agentpack-import", version: "0.1.0" }});
await client.connect(transport);
const info = client.getServerVersion();
const {{ tools }} = await client.listTools();
const result = {{ serverInfo: info, tools: tools.map(t => ({{ name: t.name, description: t.description || "" }})) }};
console.log(JSON.stringify(result));
await client.close();
process.exit(0);
"#,
        cmd = cmd,
        args_json = serde_json::to_string(args)?
    );

    let output = Command::new("node")
        .args(["--input-type=module", "-e", &script])
        .output()
        .context(
            "Failed to run node. Ensure Node.js and @modelcontextprotocol/sdk are installed.",
        )?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Introspection failed: {}",
            stderr.lines().last().unwrap_or(&stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout).context("Invalid UTF-8")?;
    let result: serde_json::Value =
        serde_json::from_str(stdout.trim()).context("Failed to parse introspection result")?;

    let server_version = result
        .get("serverInfo")
        .and_then(|s| s.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("0.1.0");

    let tools: Vec<Tool> = result
        .get("tools")
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .map(|t| Tool {
                    name: t.get("name").and_then(|n| n.as_str()).unwrap_or("").into(),
                    description: t
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .into(),
                })
                .collect()
        })
        .unwrap_or_default();

    println!("  ✓ Found {} tools:", tools.len());
    for tool in &tools {
        println!("    - {}: {}", tool.name, tool.description);
    }

    let manifest = Manifest {
        name: name.to_string(),
        version: server_version.to_string(),
        description: String::new(),
        pkg_type: "mcp-server".into(),
        transport: Some(Transport {
            transport_type: "stdio".into(),
            command: Some(cmd.to_string()),
            args: args.to_vec(),
            url: None,
        }),
        dependencies: BTreeMap::new(),
        agents: BTreeMap::new(),
        provides: None,
        requires: vec![],
        profiles: BTreeMap::new(),
        tools,
    };

    let dir = format!("packages/{}", name.replace('/', "__"));
    std::fs::create_dir_all(&dir)?;
    let path = format!("{}/agentpack.json", dir);
    std::fs::write(&path, serde_json::to_string_pretty(&manifest)?)?;

    println!("\n  → {}", path);
    println!(
        "  Run `agentpack add {}@^{}` to use it.",
        name, server_version
    );
    Ok(())
}
