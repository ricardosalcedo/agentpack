use anyhow::{bail, Result};
use std::collections::BTreeMap;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::manifest::{CredentialsFile, LockFile};

/// Tracks running services by transport type
enum RunningService {
    /// A stdio process managed directly
    Stdio(Child),
    /// A docker container identified by container ID
    Docker(String),
}

pub fn run() -> Result<()> {
    let lock = LockFile::load()?;
    let creds = CredentialsFile::load()?;

    if lock.resolved.is_empty() {
        bail!("No dependencies resolved. Run `agentpack install` first.");
    }

    let order = topo_sort(&lock)?;

    println!("Starting {} services in dependency order...\n", order.len());

    let mut services: BTreeMap<String, RunningService> = BTreeMap::new();

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

        let env_vars = creds
            .as_ref()
            .map(|c| c.resolve_env(name))
            .unwrap_or_default();

        match transport.transport_type.as_str() {
            "stdio" => {
                let cmd = match &transport.command {
                    Some(c) => c.clone(),
                    None => {
                        continue;
                    }
                };

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
                        services.insert(name.clone(), RunningService::Stdio(child));
                    }
                    Err(e) => {
                        eprintln!("  ✗ Failed to start {}: {}", name, e);
                    }
                }
            }
            "docker" => {
                if transport.args.is_empty() {
                    eprintln!(
                        "  ✗ {} — docker transport requires image name in args[0]",
                        name
                    );
                    continue;
                }

                let image = &transport.args[0];
                let extra_args = &transport.args[1..];

                println!("  🐳 ▶ {} @ {} (docker: {})", name, entry.version, image);

                let mut command = Command::new("docker");
                command.args(["run", "--rm", "-d", "-i"]);

                // Pass environment variables to the container
                for (k, v) in &env_vars {
                    command.args(["-e", &format!("{}={}", k, v)]);
                }

                command.arg(image);
                command.args(extra_args);

                match command.output() {
                    Ok(output) => {
                        if output.status.success() {
                            let container_id =
                                String::from_utf8_lossy(&output.stdout).trim().to_string();
                            println!("    Container ID: {}", container_id);
                            services.insert(name.clone(), RunningService::Docker(container_id));
                        } else {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            eprintln!(
                                "  ✗ Failed to start docker container for {}: {}",
                                name,
                                stderr.trim()
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("  ✗ Failed to run docker for {}: {}", name, e);
                    }
                }
            }
            other => {
                println!("  ⏭ {} [{}] — remote ({})", name, entry.entry_type, other);
                continue;
            }
        }

        thread::sleep(Duration::from_millis(100));
    }

    if services.is_empty() {
        println!("\nNo services to manage.");
        return Ok(());
    }

    // Set up Ctrl+C shutdown signal
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_hook = shutdown.clone();
    ctrlc::set_handler(move || {
        shutdown_hook.store(true, Ordering::SeqCst);
    })
    .ok();

    println!(
        "\n✓ {} services running. Press Ctrl+C to stop all.\n",
        services.len()
    );

    loop {
        thread::sleep(Duration::from_secs(2));

        // Check for shutdown signal
        if shutdown.load(Ordering::SeqCst) {
            println!("\nShutting down...");
            shutdown_services(&mut services);
            break;
        }

        let mut dead = vec![];
        for (name, service) in services.iter_mut() {
            match service {
                RunningService::Stdio(child) => match child.try_wait() {
                    Ok(Some(status)) => {
                        eprintln!("  ✗ {} exited ({})", name, status);
                        dead.push(name.clone());
                    }
                    Ok(None) => {}
                    Err(_) => {
                        dead.push(name.clone());
                    }
                },
                RunningService::Docker(container_id) => {
                    // Check if the container is still running
                    let output = Command::new("docker")
                        .args(["inspect", "-f", "{{.State.Running}}", container_id])
                        .output();
                    match output {
                        Ok(o) => {
                            let running = String::from_utf8_lossy(&o.stdout).trim().to_string();
                            if running != "true" {
                                eprintln!("  ✗ {} container stopped", name);
                                dead.push(name.clone());
                            }
                        }
                        Err(_) => {
                            dead.push(name.clone());
                        }
                    }
                }
            }
        }
        for name in dead {
            services.remove(&name);
        }
        if services.is_empty() {
            println!("All services have exited.");
            break;
        }
    }

    Ok(())
}

/// Shutdown all running services, stopping docker containers gracefully
fn shutdown_services(services: &mut BTreeMap<String, RunningService>) {
    for (name, service) in services.iter_mut() {
        match service {
            RunningService::Stdio(child) => {
                println!("  Stopping {}...", name);
                let _ = child.kill();
            }
            RunningService::Docker(container_id) => {
                println!("  🐳 Stopping container for {}...", name);
                let _ = Command::new("docker").args(["stop", container_id]).output();
            }
        }
    }
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
