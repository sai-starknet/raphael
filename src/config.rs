use crate::error::{Result, SaiError};
use serde::{Deserialize, Serialize};
use starknet::core::types::FieldElement;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub account_address: Option<String>,
    pub private_key: Option<String>,
    pub rpc_url: Option<String>,
    pub keystore_path: Option<PathBuf>,
    pub password: Option<String>,
}

impl AccountConfig {
    pub fn validate(&self) -> Result<()> {
        if self.account_address.is_none() {
            return Err(SaiError::MissingField("account_address".to_string()));
        }

        if self.private_key.is_none() && self.keystore_path.is_none() {
            return Err(SaiError::MissingField(
                "either private_key or keystore_path".to_string(),
            ));
        }

        if self.rpc_url.is_none() {
            return Err(SaiError::MissingField("rpc_url".to_string()));
        }

        // Validate RPC URL format
        if let Some(ref url) = self.rpc_url {
            Url::parse(url)
                .map_err(|e| SaiError::Config(format!("Invalid RPC URL '{}': {}", url, e)))?;
        }

        // Validate account address format
        if let Some(ref addr) = self.account_address {
            crate::utils::hex_to_field_element(addr)
                .map_err(|e| SaiError::Config(format!("Invalid account address: {}", e)))?;
        }

        Ok(())
    }

    pub fn get_account_address(&self) -> Result<FieldElement> {
        let addr_str = self
            .account_address
            .as_ref()
            .ok_or_else(|| SaiError::MissingField("account_address".to_string()))?;
        crate::utils::hex_to_field_element(addr_str)
    }

    pub fn get_rpc_url(&self) -> Result<Url> {
        let url_str = self
            .rpc_url
            .as_ref()
            .ok_or_else(|| SaiError::MissingField("rpc_url".to_string()))?;
        Url::parse(url_str).map_err(|e| SaiError::Config(format!("Invalid RPC URL: {}", e)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScarbConfig {
    pub package: PackageConfig,
    #[serde(default)]
    pub dependencies: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub authors: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclareConfig {
    pub name: Option<String>,
    pub sierra_path: Option<PathBuf>,
    pub casm_path: Option<PathBuf>,
}

impl DeclareConfig {
    pub fn get_sierra_path(&self, project_name: &str, tag: &str, target_path: &Path) -> PathBuf {
        self.sierra_path.clone().unwrap_or_else(|| {
            target_path.join(format!(
                "{}_{}.contract_class.json",
                project_name,
                self.name.as_ref().unwrap_or(&tag.to_string())
            ))
        })
    }

    pub fn get_casm_path(&self, project_name: &str, tag: &str, target_path: &Path) -> PathBuf {
        self.casm_path.clone().unwrap_or_else(|| {
            target_path.join(format!(
                "{}_{}.compiled_contract_class.json",
                project_name,
                self.name.as_ref().unwrap_or(&tag.to_string())
            ))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    pub class: Option<String>,
    pub class_hash: Option<String>,
    pub salt: Option<String>,
    pub unique: Option<bool>,
    pub constructor_calldata: Option<Vec<String>>,
    pub wait_for_acceptance: Option<bool>,
}

impl DeployConfig {
    pub fn get_salt(&self) -> Result<Option<FieldElement>> {
        match &self.salt {
            Some(salt_str) => Ok(Some(crate::utils::hex_to_field_element(salt_str)?)),
            None => Ok(None),
        }
    }

    pub fn get_class_hash(&self) -> Result<Option<FieldElement>> {
        match &self.class_hash {
            Some(hash_str) => Ok(Some(crate::utils::hex_to_field_element(hash_str)?)),
            None => Ok(None),
        }
    }

    pub fn get_constructor_calldata(&self) -> Result<Vec<FieldElement>> {
        match &self.constructor_calldata {
            Some(calldata_strs) => calldata_strs
                .iter()
                .map(|s| crate::utils::hex_to_field_element(s))
                .collect(),
            None => Ok(Vec::new()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub account: AccountConfig,
    #[serde(default)]
    pub declare: HashMap<String, DeclareConfig>,
    #[serde(default)]
    pub deploy: HashMap<String, DeployConfig>,
    pub network: Option<NetworkConfig>,
}

impl ProfileConfig {
    pub fn validate(&self) -> Result<()> {
        self.account.validate()?;

        // Validate declare configs
        for (tag, config) in &self.declare {
            if let Some(ref name) = config.name {
                if name.is_empty() {
                    return Err(SaiError::Config(format!(
                        "Empty name in declare config for tag '{}'",
                        tag
                    )));
                }
            }
        }

        // Validate deploy configs
        for (tag, config) in &self.deploy {
            if config.class.is_none() && config.class_hash.is_none() {
                return Err(SaiError::Config(format!(
                    "Deploy config for tag '{}' must have either 'class' or 'class_hash'",
                    tag
                )));
            }

            // Validate field elements if provided
            config.get_salt()?;
            config.get_class_hash()?;
            config.get_constructor_calldata()?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub chain_id: Option<String>,
    pub explorer_url: Option<String>,
}

/// Load and validate a TOML configuration file
pub fn load_toml<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let path_ref = path.as_ref();
    if !path_ref.exists() {
        return Err(SaiError::Config(format!(
            "Configuration file not found: {}",
            path_ref.display()
        )));
    }

    let content = std::fs::read_to_string(path_ref)?;
    let config: T = toml::from_str(&content).map_err(|e| {
        SaiError::Config(format!(
            "Failed to parse TOML file '{}': {}",
            path_ref.display(),
            e
        ))
    })?;

    Ok(config)
}

/// Load and validate a JSON configuration file
pub fn load_json<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let path_ref = path.as_ref();
    if !path_ref.exists() {
        return Err(SaiError::Config(format!(
            "JSON file not found: {}",
            path_ref.display()
        )));
    }

    let content = std::fs::read_to_string(path_ref)?;
    let config: T = serde_json::from_str(&content).map_err(|e| {
        SaiError::Config(format!(
            "Failed to parse JSON file '{}': {}",
            path_ref.display(),
            e
        ))
    })?;

    Ok(config)
}

/// Save data as pretty-printed JSON
pub fn dump_json<T: Serialize>(path: impl AsRef<Path>, data: &T) -> Result<()> {
    let content = serde_json::to_string_pretty(data).map_err(|e| SaiError::Json(e))?;

    // Create parent directories if they don't exist
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(path, content)?;
    Ok(())
}

/// Resolve a path, handling relative paths correctly
pub fn resolve_path(path: &str) -> PathBuf {
    let path_buf = PathBuf::from(path);
    if path_buf.is_absolute() {
        path_buf
    } else {
        std::env::current_dir().unwrap_or_default().join(path_buf)
    }
}
