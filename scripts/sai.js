import { Contract, CairoCustomEnum, Account, hash } from "starknet";
import {
  loadJson,
  loadToml,
  resolvePath,
  declareContract,
  deployContract,
  calculateUDCContractAddressFromHash,
} from "./stark-utils.js";
import commandLineArgs from "command-line-args";
import * as accounts from "web3-eth-accounts";

const cmdOptions = [
  { name: "profile", type: String, defaultOption: true, defaultValue: "dev" },
  { name: "password", alias: "p", type: String, defaultValue: null },
  { name: "keystore_path", alias: "k", type: String, defaultValue: null },
  { name: "account_address", alias: "A", type: String, defaultValue: null },
  { name: "rpc_url", alias: "u", type: String, defaultValue: null },
];

export const readKeystorePK = async (
  keystorePath,
  accountAddress,
  password
) => {
  let data = loadJson(keystorePath);
  data.address = accountAddress;
  return (await accounts.decrypt(data, password)).privateKey;
};

const loadAccount = async (account, cmdOptions) => {
  const accountAddress = cmdOptions.account_address || account.account_address;
  const nodeUrl = cmdOptions.rpc_url || account.rpc_url;
  let privateKey = cmdOptions.private_key || account.private_key;
  if (privateKey == null) {
    const keystorePath = cmdOptions.keystore_path || account.keystore_path;
    const password = cmdOptions.password || account.password;
    privateKey = await readKeystorePK(keystorePath, accountAddress, password);
  }
  return new Account({ nodeUrl }, accountAddress, privateKey);
};

const loadSai = (scarbToml, profileToml) => {
  return new SaiConfig(profile, profileToml.contracts, scarbToml.package.name);
};

const declareContracts = async (account, targetPath, name, contracts) => {
  let declarations = {};
  for (const [tag, contractData] of Object.entries(contracts)) {
    const contractPath = `${targetPath}/${name}_${tag}.contract_class.json`;
    const casmPath = `${targetPath}/${name}_${tag}.compiled_contract_class.json`;
    declarations[tag] = await declareContract(account, contractPath, casmPath);
  }
  return declarations;
};

const deployContracts = async (account, declarations, contracts) => {
  let deployed = {};
  for (const [tag, data] of Object.entries(contracts)) {
    const classHash = declarations[tag].class_hash;
    try {
      deployed[tag] = {
        ...data,
        ...(await deployContract(
          account,
          classHash,
          data.calldata,
          data.salt,
          data.unique
        )),
      };
    } catch (e) {
      console.error(e);
      deployed[tag] = {
        ...data,
        contract_address: calculateUDCContractAddressFromHash(
          account.address,
          classHash,
          data.salt,
          data.unique,
          data.calldata
        ),
      };
    }
  }
  return deployed;
};

export class SaiConfig {
  constructor(profile, contracts, name, account) {
    this.contracts = contracts;
    this.profile = profile;
    this.name = name;
  }
}

const cmdArgs = commandLineArgs(cmdOptions);
const profile = cmdArgs.profile || "dev";

const targetPath = resolvePath(`./target/${profile}`);
const scarb_toml = loadToml(resolvePath(`./Scarb.toml`));
const profile_toml = loadToml(resolvePath(`./sai_${profile}.toml`));

// console.log(profile_toml);
const account = await loadAccount(profile_toml.account, cmdArgs);

const sai = loadSai(scarb_toml, profile_toml);
const declarations = await declareContracts(
  account,
  targetPath,
  sai.name,
  sai.contracts.declare
);
console.log(declarations);
const deployments = await deployContracts(
  account,
  declarations,
  sai.contracts.deploy
);
console.log(deployments);
// console.log(declaredContracts);
// const deployedContracts = await deployContracts(
//   account,
//   declaredContracts,
//   sai.contracts.deploy
// );

// const contract = deployedContracts.deployed_contract;
// console.log(deployedContracts);
