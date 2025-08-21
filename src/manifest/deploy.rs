use smol_str::SmolStr;
use starknet_crypto::Felt;

pub struct DeployToml {
    pub tag: Option<SmolStr>,
    pub name: Option<SmolStr>,
    pub file_name: Option<SmolStr>,
    pub class_hash: Option<Felt>,
    pub salt: Option<Felt>,
    pub unique: Option<bool>,
    pub calldata: Option<Calldata>,
}
