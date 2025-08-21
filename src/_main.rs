mod config;
mod error;
mod sai;
mod utils;

use clap::Parser;
use starknet::{
    accounts::{Account, SingleOwnerAccount},
    providers::{jsonrpc::HttpTransport, JsonRpcClient},
    signers::{LocalWallet, SigningKey},
};
use std::path::PathBuf;

use crate::{
    config::{load_toml, resolve_path, ProfileConfig, ScarbConfig},
    error::{Result, SaiError},
    sai::SaiProject,
};

#[derive(Parser, Debug)]
#[command(author, version, about = "StarkNet deployment tool")]
struct Args {
    /// Profile to use (corresponds to sai_<profile>.toml)
    #[arg(default_value = "dev")]
    profile: String,

    /// Password for keystore (if using keystore)
    #[arg(short, long)]
    password: Option<String>,

    /// Keystore path override
    #[arg(short = 'k', long)]
    keystore_path: Option<String>,

    /// Account address override
    #[arg(short = 'A', long)]
    account_address: Option<String>,

    /// RPC URL override
    #[arg(short = 'u', long)]
    rpc_url: Option<String>,

    /// Directory path (project root)
    #[arg(short = 'd', long, default_value = ".")]
    directory_path: String,

    /// Skip declaration step
    #[arg(long)]
    skip_declare: bool,

    /// Skip deployment step
    #[arg(long)]
    skip_deploy: bool,

    /// Only declare, don't deploy
    #[arg(long)]
    declare_only: bool,

    /// Only deploy, don't declare
    #[arg(long)]
    deploy_only: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

type StarkNetAccount = SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>;

async fn create_account(
    account_config: &config::AccountConfig,
    args: &Args,
) -> Result<StarkNetAccount> {
    // Use command line overrides if provided
    let account_address = args
        .account_address
        .as_ref()
        .or(account_config.account_address.as_ref())
        .ok_or_else(|| SaiError::MissingField("account_address".to_string()))?;

    let rpc_url = args
        .rpc_url
        .as_ref()
        .or(account_config.rpc_url.as_ref())
        .ok_or_else(|| SaiError::MissingField("rpc_url".to_string()))?;

    // For now, we only support private key authentication
    // TODO: Add keystore support
    let private_key = account_config
        .private_key
        .as_ref()
        .ok_or_else(|| SaiError::MissingField("private_key".to_string()))?;

    if args.verbose {
        println!("Connecting to RPC: {}", rpc_url);
        println!("Using account: {}", account_address);
    }

    // Create provider
    let rpc_client = JsonRpcClient::new(HttpTransport::new(account_config.get_rpc_url()?));

    // Create signer
    let private_key_field = crate::utils::hex_to_field_element(private_key)?;
    let signing_key = SigningKey::from_secret_scalar(private_key_field);
    let signer = LocalWallet::from(signing_key);

    // Create account
    let account_address_field = crate::utils::hex_to_field_element(account_address)?;
    let chain_id = starknet::core::chain_id::MAINNET; // TODO: Make configurable
    let account = SingleOwnerAccount::new(rpc_client, signer, account_address_field, chain_id);

    Ok(account)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        println!("🚀 Starting SAI deployment tool");
        println!("Profile: {}", args.profile);
        println!("Directory: {}", args.directory_path);
    }

    // Validate mutually exclusive options
    if args.declare_only && args.deploy_only {
        return Err(SaiError::InvalidInput(
            "Cannot specify both --declare-only and --deploy-only".to_string(),
        ));
    }

    // Load configuration files
    let target_path = resolve_path(&format!("{}/target/{}", args.directory_path, args.profile));
    let scarb_toml_path = resolve_path(&format!("{}/Scarb.toml", args.directory_path));
    let profile_toml_path = resolve_path(&format!(
        "{}/sai_{}.toml",
        args.directory_path, args.profile
    ));

    if args.verbose {
        println!("📄 Loading configuration files:");
        println!("  Scarb.toml: {}", scarb_toml_path.display());
        println!("  Profile config: {}", profile_toml_path.display());
        println!("  Target directory: {}", target_path.display());
    }

    // Load and validate configurations
    let scarb_config: ScarbConfig = load_toml(&scarb_toml_path)?;
    let mut profile_config: ProfileConfig = load_toml(&profile_toml_path)?;

    // Validate configuration
    profile_config.validate()?;

    if args.verbose {
        println!("✅ Configuration validated successfully");
        println!("📦 Project: {}", scarb_config.package.name);
    }

    // Create StarkNet account
    let account = create_account(&profile_config.account, &args).await?;

    if args.verbose {
        println!("🔑 Account connected successfully");
    }

    // Create SAI project
    let mut sai_project = SaiProject::new(
        args.profile.clone(),
        account,
        scarb_config.package.name,
        target_path,
        profile_config,
    );

    // Load existing manifest if available
    if let Err(e) = sai_project.load_manifest() {
        if args.verbose {
            println!("⚠️  Could not load existing manifest: {}", e);
        }
    }

    // Execute deployment pipeline
    if !args.skip_declare && !args.deploy_only {
        println!("📋 Declaring contracts...");
        sai_project.declare_all_classes().await?;

        if args.verbose {
            println!("Declared {} classes", sai_project.classes.len());
        }
    }

    if !args.skip_deploy && !args.declare_only {
        println!("🚀 Deploying contracts...");
        sai_project.deploy_all_contracts().await?;

        if args.verbose {
            println!("Deployed {} contracts", sai_project.contracts.len());
        }
    }

    // Save manifest
    sai_project.save_manifest()?;

    if args.verbose {
        println!("💾 Manifest saved");
    }

    // Print summary
    println!("\n📊 Deployment Summary:");
    println!("  Profile: {}", sai_project.profile);
    println!("  Project: {}", sai_project.name);
    println!("  Classes declared: {}", sai_project.classes.len());
    println!("  Contracts deployed: {}", sai_project.contracts.len());

    if args.verbose {
        println!("\n📋 Classes:");
        for (tag, class_info) in &sai_project.classes {
            println!("  {}: {:#x}", tag, class_info.class_hash);
        }

        println!("\n🚀 Deployments:");
        for (tag, contract_info) in &sai_project.contracts {
            println!("  {}: {:#x}", tag, contract_info.contract_address);
        }
    }

    println!("\n✅ Deployment completed successfully!");
    Ok(())
}
