use anyhow::Result;
use std::path::Path;
use std::process::Command;

use crate::manifest::LockFile;
use crate::resolver;

pub fn run() -> Result<()> {
    println!("AgentPack Doctor\n");

    let mut pass = 0u32;
    let mut fail = 0u32;

    // Check node
    match Command::new("node").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("  ✓ node: {}", ver);
            pass += 1;
        }
        _ => {
            println!("  ✗ node: not found");
            fail += 1;
        }
    }

    // Check npx
    match Command::new("npx").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("  ✓ npx: {}", ver);
            pass += 1;
        }
        _ => {
            println!("  ✗ npx: not found");
            fail += 1;
        }
    }

    // Check python3
    match Command::new("python3").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("  ✓ python3: {}", ver);
            pass += 1;
        }
        _ => {
            println!("  ✗ python3: not found");
            fail += 1;
        }
    }

    // Check docker
    match Command::new("docker").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("  ✓ docker: {}", ver);
            pass += 1;
        }
        _ => {
            println!("  ✗ docker: not found");
            fail += 1;
        }
    }

    // Check agentpack.json exists
    if Path::new("agentpack.json").exists() {
        println!("  ✓ agentpack.json: found");
        pass += 1;
    } else {
        println!("  ✗ agentpack.json: not found");
        fail += 1;
    }

    // Check agentpack.lock exists
    if Path::new("agentpack.lock").exists() {
        println!("  ✓ agentpack.lock: found");
        pass += 1;
    } else {
        println!("  ✗ agentpack.lock: not found");
        fail += 1;
    }

    // Check integrity
    let integrity_ok = check_integrity();
    if integrity_ok {
        println!("  ✓ integrity: all checksums match");
        pass += 1;
    } else {
        println!("  ✗ integrity: mismatches detected");
        fail += 1;
    }

    println!("\nSummary: {} passed, {} failed", pass, fail);
    Ok(())
}

fn check_integrity() -> bool {
    let lock = match LockFile::load() {
        Ok(l) => l,
        Err(_) => return true, // No lock file, nothing to check
    };

    let mut all_ok = true;
    for (name, entry) in &lock.resolved {
        if let Some(ref expected_integrity) = entry.integrity {
            let local_path = format!("packages/{}/agentpack.json", name.replace('/', "__"));
            if Path::new(&local_path).exists() {
                if let Ok(content) = std::fs::read_to_string(&local_path) {
                    let actual = resolver::compute_integrity(&content);
                    if &actual != expected_integrity {
                        eprintln!(
                            "    ⚠ {}: integrity mismatch (expected {}, got {})",
                            name, expected_integrity, actual
                        );
                        all_ok = false;
                    }
                }
            }
        }
    }
    all_ok
}
