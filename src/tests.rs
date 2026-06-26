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
        fs::create_dir_all(&dir).expect("create test dir");
        dir
    }

    // === Manifest parsing ===

    #[test]
    fn test_manifest_roundtrip() {
        let dir = test_dir("manifest_rt");
        let path = dir.join("agentpack.json");
        let manifest = Manifest {
            name: "io.github.test/proj".into(),
            version: "1.0.0".into(),
            description: "Test".into(),
            pkg_type: "composite".into(),
            transport: None,
            dependencies: BTreeMap::from([("io.github.a/s".into(), "^1.0.0".into())]),
            agents: BTreeMap::from([("io.github.a/a".into(), AgentDep::Version("^2.0.0".into()))]),
            provides: None,
            requires: vec![],
            profiles: BTreeMap::new(),
            tools: vec![Tool {
                name: "t".into(),
                description: "".into(),
            }],
        };
        let json = serde_json::to_string_pretty(&manifest).expect("ser");
        fs::write(&path, &json).expect("write");
        let loaded: Manifest =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("deser");
        assert_eq!(loaded.name, "io.github.test/proj");
        assert_eq!(loaded.dependencies.len(), 1);
        assert_eq!(loaded.agents.len(), 1);
    }

    #[test]
    fn test_full_agent_dep() {
        let json = r#"{"name":"t","version":"1.0.0","dependencies":{},"agents":{"x":{"version":"^1.0.0","capabilities":["search","summarize"]}},"tools":[]}"#;
        let m: Manifest = serde_json::from_str(json).expect("parse");
        assert_eq!(m.agents["x"].version(), "^1.0.0");
        assert_eq!(
            m.agents["x"].required_capabilities(),
            &["search", "summarize"]
        );
    }

    #[test]
    fn test_provides_block() {
        let json = r#"{"name":"t","version":"1.0.0","type":"agent","dependencies":{},"agents":{},"provides":{"capabilities":["a","b"],"protocol":"a2a"},"tools":[]}"#;
        let m: Manifest = serde_json::from_str(json).expect("parse");
        let p = m.provides.expect("provides");
        assert_eq!(p.capabilities, vec!["a", "b"]);
        assert_eq!(p.protocol.expect("proto"), "a2a");
    }

    #[test]
    fn test_simple_agent_dep() {
        let json = r#"{"name":"t","version":"1.0.0","dependencies":{},"agents":{"x":"^1.0.0"},"tools":[]}"#;
        let m: Manifest = serde_json::from_str(json).expect("parse");
        assert_eq!(m.agents["x"].version(), "^1.0.0");
    }

    // === Profiles ===

    #[test]
    fn test_profile_filtering() {
        let json = r#"{
            "name": "t", "version": "1.0.0",
            "dependencies": {
                "io.github.a/fs": "^1.0.0",
                "io.github.b/search": "^2.0.0",
                "io.github.c/weather": "^3.0.0"
            },
            "agents": {
                "io.github.x/agent": "^1.0.0"
            },
            "profiles": {
                "minimal": { "dependencies": ["io.github.a/fs"], "agents": [] },
                "dev": { "dependencies": ["io.github.a/fs", "io.github.b/search"], "agents": ["io.github.x/agent"] },
                "prod": { "dependencies": ["io.github.a/fs", "io.github.b/search", "io.github.c/weather"], "agents": ["io.github.x/agent"] }
            },
            "tools": []
        }"#;
        let m: Manifest = serde_json::from_str(json).expect("parse");

        let minimal = m.deps_for_profile(Some("minimal"));
        assert_eq!(minimal.len(), 1);
        assert!(minimal.contains_key("io.github.a/fs"));

        let dev = m.deps_for_profile(Some("dev"));
        assert_eq!(dev.len(), 2);

        let prod = m.deps_for_profile(Some("prod"));
        assert_eq!(prod.len(), 3);

        let all = m.deps_for_profile(None);
        assert_eq!(all.len(), 3);

        let dev_agents = m.agents_for_profile(Some("dev"));
        assert_eq!(dev_agents.len(), 1);

        let minimal_agents = m.agents_for_profile(Some("minimal"));
        assert_eq!(minimal_agents.len(), 0);
    }

    // === Semantic Capability Resolution ===

    #[test]
    fn test_capability_resolution() {
        let dir = test_dir("cap_resolve");
        std::env::set_current_dir(&dir).expect("cd");

        // Create a package that provides "geocoding"
        fs::create_dir_all("packages/io.github.x__geo").expect("mkdir");
        fs::write(
            "packages/io.github.x__geo/agentpack.json",
            r#"{
            "name": "io.github.x/geo", "version": "2.0.0", "type": "mcp-server",
            "provides": {"capabilities": ["geocoding", "reverse-geocoding"]},
            "dependencies": {}, "tools": []
        }"#,
        )
        .expect("write");

        let mut manifest: Manifest = serde_json::from_str(
            r#"{
            "name": "test", "version": "1.0.0",
            "dependencies": {},
            "requires": [{"capability": "geocoding"}],
            "agents": {}, "tools": []
        }"#,
        )
        .expect("parse");

        manifest.resolve_capabilities().expect("resolve");

        assert_eq!(
            manifest.requires[0].resolved_by.as_deref(),
            Some("io.github.x/geo")
        );
        assert!(manifest.dependencies.contains_key("io.github.x/geo"));
    }

    #[test]
    fn test_capability_not_found() {
        let dir = test_dir("cap_not_found");
        std::env::set_current_dir(&dir).expect("cd");
        fs::create_dir_all("packages").expect("mkdir");

        let mut manifest: Manifest = serde_json::from_str(
            r#"{
            "name": "test", "version": "1.0.0",
            "dependencies": {},
            "requires": [{"capability": "teleportation"}],
            "agents": {}, "tools": []
        }"#,
        )
        .expect("parse");

        manifest.resolve_capabilities().expect("resolve");
        assert!(manifest.requires[0].resolved_by.is_none());
    }

    // === Resolver ===

    #[test]
    fn test_resolve_inferred() {
        let dir = test_dir("resolve_inf2");
        std::env::set_current_dir(&dir).expect("cd");
        let manifest: Manifest = serde_json::from_str(
            r#"{
            "name": "t", "version": "1.0.0",
            "dependencies": {"io.github.a/fs": "^1.2.0", "io.github.b/s": "~2.0.0"},
            "agents": {}, "tools": []
        }"#,
        )
        .expect("parse");

        let lock = resolver::resolve(&manifest, None).expect("resolve");
        assert_eq!(lock.resolved.len(), 2);
        assert_eq!(lock.resolved["io.github.a/fs"].version, "1.2.0");
        assert_eq!(lock.resolved["io.github.b/s"].version, "2.0.0");
    }

    #[test]
    fn test_resolve_with_profile() {
        let dir = test_dir("resolve_profile");
        std::env::set_current_dir(&dir).expect("cd");
        let manifest: Manifest = serde_json::from_str(
            r#"{
            "name": "t", "version": "1.0.0",
            "dependencies": {"io.github.a/fs": "^1.0.0", "io.github.b/s": "^2.0.0"},
            "profiles": {"minimal": {"dependencies": ["io.github.a/fs"], "agents": []}},
            "agents": {}, "tools": []
        }"#,
        )
        .expect("parse");

        let lock_all = resolver::resolve(&manifest, None).expect("resolve");
        assert_eq!(lock_all.resolved.len(), 2);

        let lock_min = resolver::resolve(&manifest, Some("minimal")).expect("resolve");
        assert_eq!(lock_min.resolved.len(), 1);
        assert!(lock_min.resolved.contains_key("io.github.a/fs"));
    }

    #[test]
    fn test_resolve_local() {
        let dir = test_dir("resolve_local2");
        std::env::set_current_dir(&dir).expect("cd");
        fs::create_dir_all("packages/io.github.t__s").expect("mkdir");
        fs::write(
            "packages/io.github.t__s/agentpack.json",
            r#"{
            "name":"io.github.t/s","version":"3.5.0","type":"mcp-server",
            "transport":{"type":"stdio","command":"node","args":["i.js"]},
            "dependencies":{},"tools":[]
        }"#,
        )
        .expect("write");

        let manifest: Manifest = serde_json::from_str(r#"{
            "name":"t","version":"1.0.0","dependencies":{"io.github.t/s":"^3.0.0"},"agents":{},"tools":[]
        }"#).expect("parse");

        let lock = resolver::resolve(&manifest, None).expect("resolve");
        assert_eq!(lock.resolved["io.github.t/s"].version, "3.5.0");
        assert_eq!(lock.resolved["io.github.t/s"].source.source_type, "local");
    }

    #[test]
    fn test_resolve_agent() {
        let dir = test_dir("resolve_agent2");
        std::env::set_current_dir(&dir).expect("cd");
        fs::create_dir_all("packages/io.github.t__a").expect("mkdir");
        fs::write(
            "packages/io.github.t__a/agentpack.json",
            r#"{
            "name":"io.github.t/a","version":"2.0.0","type":"agent",
            "provides":{"capabilities":["research"],"protocol":"a2a"},
            "transport":{"type":"stdio","command":"python","args":["a.py"]},
            "dependencies":{"io.github.x/s":"^1.0.0"},"agents":{},"tools":[]
        }"#,
        )
        .expect("write");

        let manifest: Manifest = serde_json::from_str(
            r#"{
            "name":"t","version":"1.0.0","dependencies":{},
            "agents":{"io.github.t/a":"^2.0.0"},"tools":[]
        }"#,
        )
        .expect("parse");

        let lock = resolver::resolve(&manifest, None).expect("resolve");
        let e = &lock.resolved["io.github.t/a"];
        assert_eq!(e.entry_type, "agent");
        assert_eq!(
            e.provides.as_ref().expect("p").capabilities,
            vec!["research"]
        );
    }

    // === Integrity ===

    #[test]
    fn test_integrity_deterministic() {
        let h1 = resolver::compute_integrity("hello");
        let h2 = resolver::compute_integrity("hello");
        assert_eq!(h1, h2);
        assert!(h1.starts_with("sha256-"));
        assert_ne!(h1, resolver::compute_integrity("world"));
    }

    // === Lock file ===

    #[test]
    fn test_lockfile_roundtrip() {
        let dir = test_dir("lock_rt2");
        let path = dir.join("agentpack.lock");
        let lock = LockFile {
            lock_version: 1,
            resolved: BTreeMap::from([(
                "io.github.t/p".into(),
                ResolvedEntry {
                    version: "1.0.0".into(),
                    entry_type: "mcp-server".into(),
                    source: Source {
                        source_type: "npm".into(),
                        package: Some("@t/p".into()),
                        version: Some("1.0.0".into()),
                    },
                    integrity: Some("sha256-abc".into()),
                    transport: Some(Transport {
                        transport_type: "stdio".into(),
                        command: Some("npx".into()),
                        args: vec!["-y".into(), "@t/p@1.0.0".into()],
                        url: None,
                    }),
                    dependencies: BTreeMap::new(),
                    agents: BTreeMap::new(),
                    provides: None,
                },
            )]),
        };
        let json = serde_json::to_string_pretty(&lock).expect("ser");
        fs::write(&path, &json).expect("write");
        let loaded: LockFile =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("deser");
        assert_eq!(loaded.resolved["io.github.t/p"].version, "1.0.0");
    }

    // === Credentials ===

    #[test]
    fn test_credentials_parsing() {
        let dir = test_dir("creds2");
        std::env::set_current_dir(&dir).expect("cd");
        fs::write("agentpack.credentials.yaml", "vaults:\n  default:\n    type: env\ncredentials:\n  x:\n    KEY:\n      vault: default\n      key: MY_KEY\n").expect("write");
        let creds = CredentialsFile::load().expect("load").expect("some");
        assert_eq!(creds.vaults["default"].vault_type, "env");
    }

    #[test]
    fn test_credentials_missing() {
        let dir = test_dir("creds_miss2");
        std::env::set_current_dir(&dir).expect("cd");
        assert!(CredentialsFile::load().expect("load").is_none());
    }

    // === Conflict detection ===

    #[test]
    fn test_conflicts() {
        let dir = test_dir("conflicts2");
        std::env::set_current_dir(&dir).expect("cd");
        fs::create_dir_all("packages/io.github.a__s").expect("mkdir");
        fs::write("packages/io.github.a__s/agentpack.json", r#"{"name":"io.github.a/s","version":"1.0.0","type":"mcp-server","dependencies":{},"tools":[{"name":"dup","description":""}]}"#).expect("w");
        fs::create_dir_all("packages/io.github.b__s").expect("mkdir");
        fs::write("packages/io.github.b__s/agentpack.json", r#"{"name":"io.github.b/s","version":"1.0.0","type":"mcp-server","dependencies":{},"tools":[{"name":"dup","description":""}]}"#).expect("w");

        let manifest: Manifest = serde_json::from_str(r#"{"name":"t","version":"1.0.0","dependencies":{"io.github.a/s":"^1.0.0","io.github.b/s":"^1.0.0"},"agents":{},"tools":[]}"#).expect("p");
        let lock = resolver::resolve(&manifest, None).expect("resolve");
        assert_eq!(lock.resolved.len(), 2);
    }
}
