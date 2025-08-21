use crate::error::{Result, SaiError};
use starknet::{accounts::Account, core::utils::get_contract_address, FieldElement};
use std::path::Path;

/// Universal Deployer Contract address on StarkNet
pub const UDC_ADDRESS: &str = "0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf";

/// Calculate contract address using the UDC (Universal Deployer Contract)
pub fn calculate_udc_contract_address(
    deployer_address: FieldElement,
    salt: FieldElement,
    class_hash: FieldElement,
    constructor_calldata: &[FieldElement],
) -> Result<FieldElement> {
    // UDC uses this formula for contract address calculation:
    let contract_address =
        get_contract_address(salt, class_hash, constructor_calldata, deployer_address);

    Ok(contract_address)
}

/// Read a contract class file (simplified for now)
pub fn read_contract_class(path: impl AsRef<Path>) -> Result<serde_json::Value> {
    let content = std::fs::read_to_string(path)?;
    let contract_class: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| SaiError::InvalidInput(format!("Invalid contract class: {}", e)))?;

    Ok(contract_class)
}

/// Declare a contract on StarkNet (placeholder implementation)
pub async fn declare_contract<A>(
    account: &A,
    sierra_path: impl AsRef<Path>,
    casm_path: impl AsRef<Path>,
) -> Result<(FieldElement, serde_json::Value)>
where
    A: Account + Sync,
{
    // Read the contract classes
    let sierra_class = read_contract_class(sierra_path)?;
    let _casm_class = read_contract_class(casm_path)?;

    // TODO: Implement actual declaration using starknet library
    // For now, return a placeholder
    let class_hash = FieldElement::from_hex_be("0x1234567890abcdef1234567890abcdef12345678")?;
    let abi = sierra_class
        .get("abi")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    println!("📋 Contract declared with class hash: {:#x}", class_hash);

    Ok((class_hash, abi))
}

/// Deploy a contract using the Universal Deployer Contract (placeholder implementation)
pub async fn deploy_contract<A>(
    account: &A,
    class_hash: FieldElement,
    constructor_calldata: Vec<FieldElement>,
    salt: FieldElement,
    unique: bool,
) -> Result<(FieldElement, FieldElement)>
where
    A: Account + Sync,
{
    // Calculate the contract address
    let deployer_address = if unique {
        account.address()
    } else {
        FieldElement::ZERO
    };

    let contract_address =
        calculate_udc_contract_address(deployer_address, salt, class_hash, &constructor_calldata)?;

    // TODO: Implement actual deployment logic
    // For now, return placeholder values
    let transaction_hash = FieldElement::from_hex_be("0x9876543210fedcba9876543210fedcba98765432")?;

    println!("🚀 Contract deployed at address: {:#x}", contract_address);
    println!("📄 Transaction hash: {:#x}", transaction_hash);

    Ok((contract_address, transaction_hash))
}

/// Generate a random salt for contract deployment
pub fn generate_random_salt() -> FieldElement {
    FieldElement::from(rand::random::<u64>())
}

/// Convert a hex string to FieldElement with better error handling
pub fn hex_to_field_element(hex: &str) -> Result<FieldElement> {
    FieldElement::from_hex_be(hex)
        .map_err(|e| SaiError::FieldElement(format!("Invalid hex string '{}': {}", hex, e)))
}
