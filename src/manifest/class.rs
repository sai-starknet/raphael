use smol_str::SmolStr;
use starknet_crypto::Felt;

pub struct ClassToml {
    pub tag: Option<SmolStr>,
    pub class_hash: Option<Felt>,
}
