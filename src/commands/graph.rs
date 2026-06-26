use crate::manifest::LockFile;
use anyhow::Result;

pub fn run() -> Result<()> {
    let lock = LockFile::load()?;

    if lock.resolved.is_empty() {
        println!("No dependencies resolved.");
        return Ok(());
    }

    println!("Dependency graph:\n");

    // Group by type
    let mcps: Vec<_> = lock
        .resolved
        .iter()
        .filter(|(_, e)| e.entry_type == "mcp-server")
        .collect();
    let agents: Vec<_> = lock
        .resolved
        .iter()
        .filter(|(_, e)| e.entry_type == "agent")
        .collect();

    if !mcps.is_empty() {
        println!("  MCP Servers:");
        for (name, entry) in &mcps {
            print_entry(name, entry, "    ");
        }
    }

    if !agents.is_empty() {
        if !mcps.is_empty() {
            println!();
        }
        println!("  Agents:");
        for (name, entry) in &agents {
            print_entry(name, entry, "    ");
            if let Some(provides) = &entry.provides {
                if !provides.capabilities.is_empty() {
                    println!("      provides: [{}]", provides.capabilities.join(", "));
                }
            }
        }
    }

    Ok(())
}

fn print_entry(name: &str, entry: &crate::manifest::ResolvedEntry, indent: &str) {
    let icon = if entry.entry_type == "agent" {
        "🤖"
    } else {
        "⚙"
    };
    println!("{}{} {} @ {}", indent, icon, name, entry.version);
    if let Some(t) = &entry.transport {
        let detail = match &t.url {
            Some(url) => format!("[{}] {}", t.transport_type, url),
            None => format!(
                "[{}] {} {}",
                t.transport_type,
                t.command.as_deref().unwrap_or("?"),
                t.args.join(" ")
            ),
        };
        println!("{}  └─ {}", indent, detail);
    }
    for (dep, ver) in &entry.dependencies {
        println!("{}  └─ needs mcp: {} @ {}", indent, dep, ver);
    }
    for (dep, ver) in &entry.agents {
        println!("{}  └─ needs agent: {} @ {}", indent, dep, ver);
    }
}
