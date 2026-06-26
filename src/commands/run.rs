use anyhow::{bail, Result};
use std::collections::BTreeMap;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

use crate::manifest::{CredentialsFile, LockFile};

pub fn run() -> Result<()> {
    let lock = LockFile::load()?;
    let creds = CredentialsFile::load()?;

    if lock.resolved.is_empty() {
        bail!("No dependencies resolved. Run `agentpack install` first.");
    }

    let order = topo_sort(&lock)?;

    println!("Starting {} services in dependency order...\n", order.len());

    let mut children: BTreeMap<String, Child> = BTreeMap::new();

    for name in &order {
        let entry = &lock.resolved[name];
        let transport = match &entry.transport {
            Some(t) => t,
            None => {
                println!(
                    "  ⏭ {} [{}] — no transport, skipping",
                    name, entry.entry_type
                );
                continue;
            }
        };

        if transport.transport_type != "stdio" {
            println!(
                "  ⏭ {} [{}] — remote ({})",
                name, entry.entry_type, transport.transport_type
            );
            continue;
        }

        let cmd = match &transport.command {
            Some(c) => c.clone(),
            None => {
                continue;
            }
        };

        let env_vars = creds
            .as_ref()
            .map(|c| c.resolve_env(name))
            .unwrap_or_default();

        let icon = if entry.entry_type == "agent" {
            "🤖"
        } else {
            "⚙"
        };
        println!("  {} ▶ {} @ {}", icon, name, entry.version);

        let mut command = Command::new(&cmd);
        command.args(&transport.args);
        for (k, v) in &env_vars {
            command.env(k, v);
        }
        command.stdin(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());

        match command.spawn() {
            Ok(child) => {
                children.insert(name.clone(), child);
            }
            Err(e) => {
                eprintln!("  ✗ Failed to start {}: {}", name, e);
            }
        }

        thread::sleep(Duration::from_millis(100));
    }

    if children.is_empty() {
        println!("\nNo stdio services to manage.");
        return Ok(());
    }

    println!(
        "\n✓ {} services running. Press Ctrl+C to stop all.\n",
        children.len()
    );

    loop {
        thread::sleep(Duration::from_secs(2));
        let mut dead = vec![];
        for (name, child) in children.iter_mut() {
            match child.try_wait() {
                Ok(Some(status)) => {
                    eprintln!("  ✗ {} exited ({})", name, status);
                    dead.push(name.clone());
                }
                Ok(None) => {}
                Err(_) => {
                    dead.push(name.clone());
                }
            }
        }
        for name in dead {
            children.remove(&name);
        }
        if children.is_empty() {
            println!("All services have exited.");
            break;
        }
    }

    Ok(())
}

fn topo_sort(lock: &LockFile) -> Result<Vec<String>> {
    let mut order = vec![];
    let mut visited: BTreeMap<String, bool> = BTreeMap::new();
    for name in lock.resolved.keys() {
        if !visited.contains_key(name) {
            topo_visit(name, lock, &mut visited, &mut order)?;
        }
    }
    Ok(order)
}

fn topo_visit(
    name: &str,
    lock: &LockFile,
    visited: &mut BTreeMap<String, bool>,
    order: &mut Vec<String>,
) -> Result<()> {
    if let Some(&in_progress) = visited.get(name) {
        if in_progress {
            bail!("Circular dependency involving '{}'", name);
        }
        return Ok(());
    }
    visited.insert(name.to_string(), true);
    if let Some(entry) = lock.resolved.get(name) {
        // Visit MCP deps first
        for dep in entry.dependencies.keys() {
            if lock.resolved.contains_key(dep) {
                topo_visit(dep, lock, visited, order)?;
            }
        }
        // Then agent deps
        for dep in entry.agents.keys() {
            if lock.resolved.contains_key(dep) {
                topo_visit(dep, lock, visited, order)?;
            }
        }
    }
    visited.insert(name.to_string(), false);
    order.push(name.to_string());
    Ok(())
}
