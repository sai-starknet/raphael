use std::collections::BTreeMap;

use semver::{Version, VersionReq};

use crate::manifest::{contract::ContractToml, declare::DeclareToml, deploy::DeployToml};

pub mod calldata;
pub mod class;
pub mod contract;
pub mod declare;
pub mod deploy;
pub mod tag;
struct ElektraToml {
    pub version: Option<VersionReq>,
    pub classes: Option<BTreeMap<Tag, ClassToml>>,
    pub contracts: Option<BTreeMap<Tag, ContractToml>>,
    pub deploys: Option<BTreeMap<Tag, DeployToml>>,
    pub declares: Option<BTreeMap<Tag, DeclareToml>>,
}
