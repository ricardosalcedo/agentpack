use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use anyhow::{Context, Result};

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
    pub tools: Vec<Tool>,
}

fn default_type() -> String { "composite".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AgentDep {
    /// Simple: just a version string
    Version(String),
    /// Full: version + metadata
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

fn default_vault() -> String { "default".into() }

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
}

impl CredentialsFile {
    pub fn load() -> Result<Option<Self>> {
        if !Path::new(CREDENTIALS_FILE).exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(CREDENTIALS_FILE)
            .context("Failed to read agentpack.credentials.yaml")?;
        let creds: Self = serde_yaml::from_str(&content)
            .context("Invalid agentpack.credentials.yaml")?;
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
