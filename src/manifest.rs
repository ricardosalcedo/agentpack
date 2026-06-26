use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "type", default = "default_type")]
    pub pkg_type: String,
    #[serde(default)]
    pub transport: Option<Transport>,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
    #[serde(default)]
    pub agents: BTreeMap<String, AgentDep>,
    #[serde(default)]
    pub provides: Option<Provides>,
    #[serde(default)]
    pub requires: Vec<CapabilityRequirement>,
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
    #[serde(default)]
    pub tools: Vec<Tool>,
}

fn default_type() -> String {
    "composite".into()
}

/// Semantic capability requirement — resolve by what you need, not by name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequirement {
    pub capability: String,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub resolved_by: Option<String>,
}

/// Workspace profile — subset of dependencies for different contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub agents: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AgentDep {
    Version(String),
    Full {
        version: String,
        #[serde(default)]
        source: Option<String>,
        #[serde(default)]
        capabilities: Vec<String>,
    },
}

impl AgentDep {
    pub fn version(&self) -> &str {
        match self {
            AgentDep::Version(v) => v,
            AgentDep::Full { version, .. } => version,
        }
    }
    pub fn source(&self) -> Option<&str> {
        match self {
            AgentDep::Version(_) => None,
            AgentDep::Full { source, .. } => source.as_deref(),
        }
    }
    pub fn required_capabilities(&self) -> &[String] {
        match self {
            AgentDep::Version(_) => &[],
            AgentDep::Full { capabilities, .. } => capabilities,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provides {
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub protocol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transport {
    #[serde(rename = "type")]
    pub transport_type: String,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockFile {
    #[serde(rename = "lockVersion")]
    pub lock_version: u32,
    pub resolved: BTreeMap<String, ResolvedEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolvedEntry {
    pub version: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub source: Source,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<Transport>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub agents: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provides: Option<Provides>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(default)]
    pub package: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

// Credentials config
#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialsFile {
    #[serde(default)]
    pub vaults: BTreeMap<String, Vault>,
    #[serde(default)]
    pub credentials: BTreeMap<String, BTreeMap<String, CredentialRef>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vault {
    #[serde(rename = "type")]
    pub vault_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialRef {
    #[serde(default = "default_vault")]
    pub vault: String,
    pub key: String,
}

fn default_vault() -> String {
    "default".into()
}

const MANIFEST_FILE: &str = "agentpack.json";
const LOCK_FILE: &str = "agentpack.lock";
const CREDENTIALS_FILE: &str = "agentpack.credentials.yaml";

impl Manifest {
    pub fn load() -> Result<Self> {
        let content = std::fs::read_to_string(MANIFEST_FILE)
            .context("No agentpack.json found. Run `agentpack init` first.")?;
        serde_json::from_str(&content).context("Invalid agentpack.json")
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(MANIFEST_FILE, json)?;
        Ok(())
    }

    pub fn exists() -> bool {
        Path::new(MANIFEST_FILE).exists()
    }

    /// Get dependencies filtered by profile (None = all)
    pub fn deps_for_profile(&self, profile: Option<&str>) -> BTreeMap<String, String> {
        match profile {
            None => self.dependencies.clone(),
            Some(p) => {
                if let Some(prof) = self.profiles.get(p) {
                    self.dependencies
                        .iter()
                        .filter(|(k, _)| prof.dependencies.contains(k))
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                } else {
                    self.dependencies.clone()
                }
            }
        }
    }

    /// Get agents filtered by profile
    pub fn agents_for_profile(&self, profile: Option<&str>) -> BTreeMap<String, AgentDep> {
        match profile {
            None => self.agents.clone(),
            Some(p) => {
                if let Some(prof) = self.profiles.get(p) {
                    self.agents
                        .iter()
                        .filter(|(k, _)| prof.agents.contains(k))
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                } else {
                    self.agents.clone()
                }
            }
        }
    }

    /// Resolve semantic capability requirements against available packages
    pub fn resolve_capabilities(&mut self) -> Result<()> {
        for req in &mut self.requires {
            if req.resolved_by.is_some() {
                continue;
            }
            // Search local packages for one that provides this capability
            let packages_dir = Path::new("packages");
            if !packages_dir.exists() {
                continue;
            }
            for entry in std::fs::read_dir(packages_dir)? {
                let entry = entry?;
                let manifest_path = entry.path().join("agentpack.json");
                if !manifest_path.exists() {
                    continue;
                }
                let content = std::fs::read_to_string(&manifest_path)?;
                if let Ok(m) = serde_json::from_str::<Manifest>(&content) {
                    let provides_cap = m
                        .provides
                        .as_ref()
                        .map(|p| p.capabilities.contains(&req.capability))
                        .unwrap_or(false);
                    if provides_cap {
                        req.resolved_by = Some(m.name.clone());
                        // Add to dependencies if not already present
                        if !self.dependencies.contains_key(&m.name) {
                            self.dependencies
                                .insert(m.name.clone(), format!("^{}", m.version));
                        }
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

impl LockFile {
    pub fn load() -> Result<Self> {
        let content = std::fs::read_to_string(LOCK_FILE)
            .context("No agentpack.lock found. Run `agentpack install` first.")?;
        serde_json::from_str(&content).context("Invalid agentpack.lock")
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(LOCK_FILE, json)?;
        Ok(())
    }

    /// Filter lock file to only include entries from a profile
    pub fn filter_by_profile(&self, manifest: &Manifest, profile: &str) -> Self {
        let deps = manifest.deps_for_profile(Some(profile));
        let agents = manifest.agents_for_profile(Some(profile));
        let allowed: std::collections::HashSet<&String> =
            deps.keys().chain(agents.keys()).collect();

        LockFile {
            lock_version: self.lock_version,
            resolved: self
                .resolved
                .iter()
                .filter(|(k, _)| allowed.contains(k))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }
}

impl Clone for ResolvedEntry {
    fn clone(&self) -> Self {
        let json = serde_json::to_string(self).expect("serialize");
        serde_json::from_str(&json).expect("deserialize")
    }
}

impl CredentialsFile {
    pub fn load() -> Result<Option<Self>> {
        if !Path::new(CREDENTIALS_FILE).exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(CREDENTIALS_FILE)
            .context("Failed to read agentpack.credentials.yaml")?;
        let creds: Self =
            serde_yaml::from_str(&content).context("Invalid agentpack.credentials.yaml")?;
        Ok(Some(creds))
    }

    pub fn resolve_env(&self, package: &str) -> BTreeMap<String, String> {
        let mut env = BTreeMap::new();
        if let Some(creds) = self.credentials.get(package) {
            for (env_var, cred_ref) in creds {
                if let Some(vault) = self.vaults.get(&cred_ref.vault) {
                    if vault.vault_type == "env" {
                        if let Ok(val) = std::env::var(&cred_ref.key) {
                            env.insert(env_var.clone(), val);
                        }
                    }
                }
            }
        }
        env
    }
}
