use crate::{
    config::{DeclareConfig, DeployConfig, ProfileConfig},
    error::{Result, SaiError},
    utils,
};
use serde::{Deserialize, Serialize};
use starknet::{
    accounts::Account,
    core::types::{contract::SierraClass, Felt},
    providers::Provider,
};
use std::collections::HashMap;
use std::path::PathBuf;

/// Information about a declared contract class
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    pub class_hash: Felt,
    pub sierra_class: Option<SierraClass>,
    pub declared_at: Option<u64>, // Block number when declared
}

/// Information about a deployed contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub contract_address: Felt,
    pub class_hash: Felt,
    pub class_tag: Option<String>,
    pub deployed_at: Option<u64>, // Block number when deployed
}

/// Information about a contract deployment transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub contract_address: Felt,
    pub deployer_address: Felt,
    pub transaction_hash: Felt,
    pub salt: Felt,
    pub class_hash: Felt,
    pub constructor_calldata: Vec<Felt>,
    pub unique: bool,
    pub deployed_at: Option<u64>,
}

/// Information about a contract declaration transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclarationInfo {
    pub class_hash: Felt,
    pub transaction_hash: Felt,
    pub declared_at: Option<u64>,
    pub sierra_class: Option<SierraClass>,
}

/// Data required to deploy a contract
#[derive(Debug, Clone)]
pub struct DeployContractData {
    pub tag: Option<String>,
    pub class_tag: Option<String>,
    pub class_hash: Option<Felt>,
    pub salt: Option<Felt>,
    pub unique: bool,
    pub constructor_calldata: Vec<Felt>,
}

impl DeployContractData {
    pub fn new() -> Self {
        Self {
            tag: None,
            class_tag: None,
            class_hash: None,
            salt: None,
            unique: false,
            constructor_calldata: Vec::new(),
        }
    }

    pub fn with_tag(mut self, tag: String) -> Self {
        self.tag = Some(tag);
        self
    }

    pub fn with_class_tag(mut self, class_tag: String) -> Self {
        self.class_tag = Some(class_tag);
        self
    }

    pub fn with_class_hash(mut self, class_hash: Felt) -> Self {
        self.class_hash = Some(class_hash);
        self
    }

    pub fn with_salt(mut self, salt: Felt) -> Self {
        self.salt = Some(salt);
        self
    }

    pub fn with_unique(mut self, unique: bool) -> Self {
        self.unique = unique;
        self
    }

    pub fn with_constructor_calldata(mut self, calldata: Vec<Felt>) -> Self {
        self.constructor_calldata = calldata;
        self
    }
}

/// Main project structure for managing StarkNet contract deployments
pub struct SaiProject<A: Account + Sync> {
    pub name: String,
    pub profile: String,
    pub account: A,
    pub target_path: PathBuf,

    // Configuration
    pub declare_configs: HashMap<String, DeclareConfig>,
    pub deploy_configs: HashMap<String, DeployConfig>,

    // Runtime state
    pub declarations: HashMap<String, DeclarationInfo>,
    pub deployments: HashMap<String, DeploymentInfo>,
    pub classes: HashMap<String, ClassInfo>,
    pub contracts: HashMap<String, ContractInfo>,
}

impl<A: Account + Sync> SaiProject<A> {
    pub fn new(
        profile: String,
        account: A,
        name: String,
        target_path: PathBuf,
        profile_config: ProfileConfig,
    ) -> Self {
        Self {
            name,
            profile,
            account,
            target_path,
            declare_configs: profile_config.declare,
            deploy_configs: profile_config.deploy,
            declarations: HashMap::new(),
            deployments: HashMap::new(),
            classes: HashMap::new(),
            contracts: HashMap::new(),
        }
    }

    /// Declare a single contract class
    pub async fn declare_class(&mut self, tag: &str) -> Result<&DeclarationInfo>
    where
        A::SignError: 'static,
    {
        let config = self
            .declare_configs
            .get(tag)
            .ok_or_else(|| SaiError::Config(format!("No declare config found for tag '{}'", tag)))?
            .clone();

        let sierra_path = config.get_sierra_path(&self.name, tag, &self.target_path);
        let casm_path = config.get_casm_path(&self.name, tag, &self.target_path);

        // Check if files exist
        if !sierra_path.exists() {
            return Err(SaiError::Config(format!(
                "Sierra file not found: {}",
                sierra_path.display()
            )));
        }
        if !casm_path.exists() {
            return Err(SaiError::Config(format!(
                "CASM file not found: {}",
                casm_path.display()
            )));
        }

        println!("Declaring contract '{}' from files:", tag);
        println!("  Sierra: {}", sierra_path.display());
        println!("  CASM:   {}", casm_path.display());

        // Declare the contract
        let (class_hash, sierra_class) =
            utils::declare_contract(&self.account, &sierra_path, &casm_path).await?;

        println!(
            "✅ Contract '{}' declared with class hash: {:#x}",
            tag, class_hash
        );

        // Store declaration info
        let declaration_info = DeclarationInfo {
            class_hash,
            transaction_hash: Felt::ZERO, // TODO: Get actual tx hash
            declared_at: None,            // TODO: Get block number
            sierra_class: Some(sierra_class.clone()),
        };

        let class_info = ClassInfo {
            class_hash,
            sierra_class: Some(sierra_class),
            declared_at: None,
        };

        self.declarations.insert(tag.to_string(), declaration_info);
        self.classes.insert(tag.to_string(), class_info);

        Ok(self.declarations.get(tag).unwrap())
    }

    /// Declare all contracts defined in the configuration
    pub async fn declare_all_classes(&mut self) -> Result<()>
    where
        A::SignError: 'static,
    {
        let tags: Vec<String> = self.declare_configs.keys().cloned().collect();

        for tag in tags {
            self.declare_class(&tag).await?;
        }

        println!("✅ All contracts declared successfully");
        Ok(())
    }

    /// Deploy a single contract
    pub async fn deploy_contract(&mut self, data: DeployContractData) -> Result<&DeploymentInfo>
    where
        A::SignError: 'static,
    {
        // Determine class hash
        let class_hash = if let Some(hash) = data.class_hash {
            hash
        } else if let Some(ref class_tag) = data.class_tag {
            self.classes
                .get(class_tag)
                .ok_or_else(|| SaiError::Config(format!("Class '{}' not found", class_tag)))?
                .class_hash
        } else {
            return Err(SaiError::Config(
                "No class_hash or class_tag specified".to_string(),
            ));
        };

        // Generate salt if not provided
        let salt = data.salt.unwrap_or_else(utils::generate_random_salt);

        // Determine deployment tag
        let tag = data
            .tag
            .unwrap_or_else(|| format!("contract_{}", hex::encode(class_hash.to_bytes_be())));

        println!(
            "Deploying contract '{}' with class hash: {:#x}",
            tag, class_hash
        );

        // Deploy the contract
        let (contract_address, transaction_hash) = utils::deploy_contract(
            &self.account,
            class_hash,
            data.constructor_calldata.clone(),
            salt,
            data.unique,
        )
        .await?;

        println!(
            "✅ Contract '{}' deployed at address: {:#x}",
            tag, contract_address
        );

        // Store deployment info
        let deployment_info = DeploymentInfo {
            contract_address,
            deployer_address: self.account.address(),
            transaction_hash,
            salt,
            class_hash,
            constructor_calldata: data.constructor_calldata,
            unique: data.unique,
            deployed_at: None, // TODO: Get block number
        };

        let contract_info = ContractInfo {
            contract_address,
            class_hash,
            class_tag: data.class_tag,
            deployed_at: None,
        };

        self.deployments.insert(tag.clone(), deployment_info);
        self.contracts.insert(tag.clone(), contract_info);

        Ok(self.deployments.get(&tag).unwrap())
    }

    /// Deploy contracts from configuration
    pub async fn deploy_contract_from_config(&mut self, tag: &str) -> Result<&DeploymentInfo>
    where
        A::SignError: 'static,
    {
        let config = self
            .deploy_configs
            .get(tag)
            .ok_or_else(|| SaiError::Config(format!("No deploy config found for tag '{}'", tag)))?
            .clone();

        let mut deploy_data = DeployContractData::new()
            .with_tag(tag.to_string())
            .with_unique(config.unique.unwrap_or(false));

        // Set class information
        if let Some(class_tag) = config.class {
            deploy_data = deploy_data.with_class_tag(class_tag);
        } else if let Some(class_hash) = config.get_class_hash()? {
            deploy_data = deploy_data.with_class_hash(class_hash);
        }

        // Set salt if provided
        if let Some(salt) = config.get_salt()? {
            deploy_data = deploy_data.with_salt(salt);
        }

        // Set constructor calldata
        let constructor_calldata = config.get_constructor_calldata()?;
        deploy_data = deploy_data.with_constructor_calldata(constructor_calldata);

        self.deploy_contract(deploy_data).await
    }

    /// Deploy all contracts defined in the configuration
    pub async fn deploy_all_contracts(&mut self) -> Result<()>
    where
        A::SignError: 'static,
    {
        let tags: Vec<String> = self.deploy_configs.keys().cloned().collect();

        for tag in tags {
            self.deploy_contract_from_config(&tag).await?;
        }

        println!("✅ All contracts deployed successfully");
        Ok(())
    }

    /// Get contract information by tag
    pub fn get_contract(&self, tag: &str) -> Option<&ContractInfo> {
        self.contracts.get(tag)
    }

    /// Get class information by tag
    pub fn get_class(&self, tag: &str) -> Option<&ClassInfo> {
        self.classes.get(tag)
    }

    /// Get deployment information by tag
    pub fn get_deployment(&self, tag: &str) -> Option<&DeploymentInfo> {
        self.deployments.get(tag)
    }

    /// Save the current state to a manifest file
    pub fn save_manifest(&self) -> Result<()> {
        let manifest = serde_json::json!({
            "profile": self.profile,
            "project_name": self.name,
            "declarations": self.declarations,
            "deployments": self.deployments,
            "classes": self.classes,
            "contracts": self.contracts,
            "generated_at": chrono::Utc::now().timestamp(),
        });

        let manifest_path = self
            .target_path
            .join(format!("manifest_{}.json", self.profile));
        crate::config::dump_json(manifest_path, &manifest)?;

        Ok(())
    }

    /// Load state from a manifest file
    pub fn load_manifest(&mut self) -> Result<()> {
        let manifest_path = self
            .target_path
            .join(format!("manifest_{}.json", self.profile));

        if !manifest_path.exists() {
            return Ok(()); // No manifest to load
        }

        let manifest: serde_json::Value = crate::config::load_json(&manifest_path)?;

        // Load declarations
        if let Some(declarations) = manifest.get("declarations") {
            self.declarations = serde_json::from_value(declarations.clone())?;
        }

        // Load deployments
        if let Some(deployments) = manifest.get("deployments") {
            self.deployments = serde_json::from_value(deployments.clone())?;
        }

        // Load classes
        if let Some(classes) = manifest.get("classes") {
            self.classes = serde_json::from_value(classes.clone())?;
        }

        // Load contracts
        if let Some(contracts) = manifest.get("contracts") {
            self.contracts = serde_json::from_value(contracts.clone())?;
        }

        println!("📁 Loaded manifest from {}", manifest_path.display());
        Ok(())
    }
}
