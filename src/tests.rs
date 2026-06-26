#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    use crate::manifest::*;
    use crate::resolver;

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("agentpack_test_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    // === Manifest parsing tests (no filesystem needed) ===

    #[test]
    fn test_manifest_roundtrip() {
        let dir = test_dir("manifest_roundtrip");
        let path = dir.join("agentpack.json");

        let manifest = Manifest {
            name: "io.github.test/my-project".into(),
            version: "1.0.0".into(),
            description: "Test".into(),
            pkg_type: "composite".into(),
            transport: None,
            dependencies: BTreeMap::from([("io.github.a/server".into(), "^1.0.0".into())]),
            agents: BTreeMap::from([(
                "io.github.a/agent".into(),
                AgentDep::Version("^2.0.0".into()),
            )]),
            provides: None,
            tools: vec![Tool {
                name: "test_tool".into(),
                description: "A tool".into(),
            }],
        };

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(&path, &json).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let loaded: Manifest = serde_json::from_str(&content).unwrap();

        assert_eq!(loaded.name, "io.github.test/my-project");
        assert_eq!(loaded.dependencies.len(), 1);
        assert_eq!(loaded.agents.len(), 1);
        assert_eq!(loaded.tools.len(), 1);
    }

    #[test]
    fn test_manifest_with_full_agent_dep() {
        let json = r#"{
            "name": "test",
            "version": "1.0.0",
            "dependencies": {},
            "agents": {
                "io.github.x/agent": {
                    "version": "^1.0.0",
                    "capabilities": ["search", "summarize"]
                }
            },
            "tools": []
        }"#;
        let m: Manifest = serde_json::from_str(json).unwrap();
        let dep = &m.agents["io.github.x/agent"];
        assert_eq!(dep.version(), "^1.0.0");
        assert_eq!(dep.required_capabilities(), &["search", "summarize"]);
    }

    #[test]
    fn test_manifest_with_provides() {
        let json = r#"{
            "name": "test-agent",
            "version": "1.0.0",
            "type": "agent",
            "dependencies": {},
            "agents": {},
            "provides": {
                "capabilities": ["web-research", "summarization"],
                "protocol": "a2a"
            },
            "tools": []
        }"#;
        let m: Manifest = serde_json::from_str(json).unwrap();
        let provides = m.provides.unwrap();
        assert_eq!(provides.capabilities, vec!["web-research", "summarization"]);
        assert_eq!(provides.protocol.unwrap(), "a2a");
    }

    #[test]
    fn test_agent_dep_simple_version() {
        let json = r#"{"name":"t","version":"1.0.0","dependencies":{},"agents":{"x":"^1.0.0"},"tools":[]}"#;
        let m: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.agents["x"].version(), "^1.0.0");
        assert!(m.agents["x"].source().is_none());
        assert!(m.agents["x"].required_capabilities().is_empty());
    }

    // === Resolver tests (need temp dirs) ===

    #[test]
    fn test_resolve_inferred_packages() {
        let dir = test_dir("resolve_inferred");
        std::env::set_current_dir(&dir).unwrap();

        let manifest = Manifest {
            name: "test".into(),
            version: "1.0.0".into(),
            description: "".into(),
            pkg_type: "composite".into(),
            transport: None,
            dependencies: BTreeMap::from([
                ("io.github.anthropic/filesystem".into(), "^1.2.0".into()),
                ("io.github.stripe/payments".into(), "~2.0.0".into()),
            ]),
            agents: BTreeMap::new(),
            provides: None,
            tools: vec![],
        };

        let lock = resolver::resolve(&manifest).unwrap();
        assert_eq!(lock.resolved.len(), 2);
        assert_eq!(
            lock.resolved["io.github.anthropic/filesystem"].version,
            "1.2.0"
        );
        assert_eq!(
            lock.resolved["io.github.anthropic/filesystem"].entry_type,
            "mcp-server"
        );
        assert_eq!(lock.resolved["io.github.stripe/payments"].version, "2.0.0");
    }

    #[test]
    fn test_resolve_local_package() {
        let dir = test_dir("resolve_local");
        std::env::set_current_dir(&dir).unwrap();

        fs::create_dir_all("packages/io.github.test__server").unwrap();
        fs::write(
            "packages/io.github.test__server/agentpack.json",
            r#"{
            "name": "io.github.test/server",
            "version": "3.5.0",
            "type": "mcp-server",
            "transport": {"type": "stdio", "command": "node", "args": ["index.js"]},
            "dependencies": {},
            "tools": [{"name": "do_thing", "description": "does a thing"}]
        }"#,
        )
        .unwrap();

        let manifest = Manifest {
            name: "test".into(),
            version: "1.0.0".into(),
            description: "".into(),
            pkg_type: "composite".into(),
            transport: None,
            dependencies: BTreeMap::from([("io.github.test/server".into(), "^3.0.0".into())]),
            agents: BTreeMap::new(),
            provides: None,
            tools: vec![],
        };

        let lock = resolver::resolve(&manifest).unwrap();
        let entry = &lock.resolved["io.github.test/server"];
        assert_eq!(entry.version, "3.5.0");
        assert_eq!(entry.source.source_type, "local");
        assert_eq!(
            entry.transport.as_ref().unwrap().command.as_deref(),
            Some("node")
        );
    }

    #[test]
    fn test_resolve_agent_dep() {
        let dir = test_dir("resolve_agent");
        std::env::set_current_dir(&dir).unwrap();

        fs::create_dir_all("packages/io.github.test__agent").unwrap();
        fs::write(
            "packages/io.github.test__agent/agentpack.json",
            r#"{
            "name": "io.github.test/agent",
            "version": "2.0.0",
            "type": "agent",
            "transport": {"type": "stdio", "command": "python", "args": ["agent.py"]},
            "provides": {"capabilities": ["research"], "protocol": "a2a"},
            "dependencies": {"io.github.x/search": "^1.0.0"},
            "agents": {},
            "tools": []
        }"#,
        )
        .unwrap();

        let manifest = Manifest {
            name: "test".into(),
            version: "1.0.0".into(),
            description: "".into(),
            pkg_type: "composite".into(),
            transport: None,
            dependencies: BTreeMap::new(),
            agents: BTreeMap::from([(
                "io.github.test/agent".into(),
                AgentDep::Version("^2.0.0".into()),
            )]),
            provides: None,
            tools: vec![],
        };

        let lock = resolver::resolve(&manifest).unwrap();
        let entry = &lock.resolved["io.github.test/agent"];
        assert_eq!(entry.entry_type, "agent");
        assert_eq!(entry.version, "2.0.0");
        assert_eq!(entry.dependencies["io.github.x/search"], "^1.0.0");
        assert_eq!(
            entry.provides.as_ref().unwrap().capabilities,
            vec!["research"]
        );
    }

    // === Conflict detection ===

    #[test]
    fn test_conflict_detection() {
        let dir = test_dir("conflict_detection");
        std::env::set_current_dir(&dir).unwrap();

        fs::create_dir_all("packages/io.github.a__server").unwrap();
        fs::write(
            "packages/io.github.a__server/agentpack.json",
            r#"{
            "name": "io.github.a/server", "version": "1.0.0",
            "type": "mcp-server", "dependencies": {},
            "tools": [{"name": "read_file", "description": ""}]
        }"#,
        )
        .unwrap();

        fs::create_dir_all("packages/io.github.b__server").unwrap();
        fs::write("packages/io.github.b__server/agentpack.json", r#"{
            "name": "io.github.b/server", "version": "1.0.0",
            "type": "mcp-server", "dependencies": {},
            "tools": [{"name": "read_file", "description": ""}, {"name": "unique_tool", "description": ""}]
        }"#).unwrap();

        let manifest = Manifest {
            name: "test".into(),
            version: "1.0.0".into(),
            description: "".into(),
            pkg_type: "composite".into(),
            transport: None,
            dependencies: BTreeMap::from([
                ("io.github.a/server".into(), "^1.0.0".into()),
                ("io.github.b/server".into(), "^1.0.0".into()),
            ]),
            agents: BTreeMap::new(),
            provides: None,
            tools: vec![],
        };

        // resolve succeeds (conflict is a warning, not fatal)
        let lock = resolver::resolve(&manifest).unwrap();
        assert_eq!(lock.resolved.len(), 2);
    }

    // === Integrity tests ===

    #[test]
    fn test_integrity_hash_deterministic() {
        let h1 = resolver::compute_integrity("hello world");
        let h2 = resolver::compute_integrity("hello world");
        assert_eq!(h1, h2);
        assert!(h1.starts_with("sha256-"));

        let h3 = resolver::compute_integrity("different content");
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_lockfile_has_integrity() {
        let dir = test_dir("lockfile_integrity");
        std::env::set_current_dir(&dir).unwrap();

        let manifest = Manifest {
            name: "test".into(),
            version: "1.0.0".into(),
            description: "".into(),
            pkg_type: "composite".into(),
            transport: None,
            dependencies: BTreeMap::from([("io.github.x/pkg".into(), "^1.0.0".into())]),
            agents: BTreeMap::new(),
            provides: None,
            tools: vec![],
        };

        let lock = resolver::resolve(&manifest).unwrap();
        for (_, entry) in &lock.resolved {
            assert!(entry.integrity.is_some());
        }
    }

    // === Lock file tests ===

    #[test]
    fn test_lockfile_roundtrip() {
        let dir = test_dir("lockfile_roundtrip");
        let path = dir.join("agentpack.lock");

        let lock = LockFile {
            lock_version: 1,
            resolved: BTreeMap::from([(
                "io.github.test/pkg".into(),
                ResolvedEntry {
                    version: "1.0.0".into(),
                    entry_type: "mcp-server".into(),
                    source: Source {
                        source_type: "npm".into(),
                        package: Some("@test/pkg".into()),
                        version: Some("1.0.0".into()),
                    },
                    integrity: Some("sha256-abc".into()),
                    transport: Some(Transport {
                        transport_type: "stdio".into(),
                        command: Some("npx".into()),
                        args: vec!["-y".into(), "@test/pkg@1.0.0".into()],
                        url: None,
                    }),
                    dependencies: BTreeMap::new(),
                    agents: BTreeMap::new(),
                    provides: None,
                },
            )]),
        };

        let json = serde_json::to_string_pretty(&lock).unwrap();
        fs::write(&path, &json).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let loaded: LockFile = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded.lock_version, 1);
        assert_eq!(loaded.resolved["io.github.test/pkg"].version, "1.0.0");
        assert_eq!(
            loaded.resolved["io.github.test/pkg"].entry_type,
            "mcp-server"
        );
    }

    // === Credentials tests ===

    #[test]
    fn test_credentials_parsing() {
        let dir = test_dir("creds_parsing");
        std::env::set_current_dir(&dir).unwrap();

        fs::write(
            "agentpack.credentials.yaml",
            r#"
vaults:
  default:
    type: env
credentials:
  io.github.stripe/payments:
    STRIPE_KEY:
      vault: default
      key: STRIPE_API_KEY
"#,
        )
        .unwrap();

        let creds = CredentialsFile::load().unwrap().unwrap();
        assert_eq!(creds.vaults["default"].vault_type, "env");
        assert!(creds.credentials.contains_key("io.github.stripe/payments"));
    }

    #[test]
    fn test_credentials_missing_file_returns_none() {
        let dir = test_dir("creds_missing");
        std::env::set_current_dir(&dir).unwrap();
        let creds = CredentialsFile::load().unwrap();
        assert!(creds.is_none());
    }
}
