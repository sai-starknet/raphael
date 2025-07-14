import { Contract, CairoCustomEnum, Account, hash, stark } from "starknet";
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
  { name: "directory_path", alias: "d", type: String, defaultValue: "." },
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

const loadSai = (profile, account, targetPath, scarbToml, profileToml) => {
  return new SaiProject(
    profile,
    account,
    scarbToml.package.name,
    targetPath,
    profileToml
  );
};

const declareContracts = async (
  account,
  targetPath,
  projectName,
  contracts
) => {
  let declarations = {};
  for (const { tag, contract_name } of contracts) {
    const name = contract_name || tag;
    const contractPath = `${targetPath}/${projectName}_${name}.contract_class.json`;
    const casmPath = `${targetPath}/${projectName}_${name}.compiled_contract_class.json`;
    declarations[tag] = await declareContract(account, contractPath, casmPath);
  }
  return declarations;
};

const parseDeployContractData = (data) => {};

const deployContracts = async (account, declarations, contracts) => {
  let deployed = {};
  for (const { tag, contract_tag, salt, unique, calldata } of contracts) {
    const { classHash, abi } = declarations[contract_tag];

    const contractAddress = calculateUDCContractAddressFromHash(
      account.address,
      classHash,
      salt,
      unique,
      calldata
    );

    try {
      deployed[tag] = {
        tag,
        contract_tag,
        ...(await deployContract(account, classHash, calldata, salt, unique)),
      };
    } catch (e) {
      console.error(e);
      deployed[tag] = {
        tag,
        contract_tag,
        salt,
        unique,
        calldata,
        contract_address: calculateUDCContractAddressFromHash(
          account.address,
          classHash,
          salt,
          unique,
          calldata
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

export class SaiProject {
  constructor(
    profile,
    account,
    name,
    targetPath,
    { declare, deploy, classes, contracts },
    transactionDetails
  ) {
    this.name = name;
    this.profile = profile;
    this.declarations = {};
    this.deployments = {};
    this.declare = declare || {};
    this.deploy = deploy || {};
    this.classes = classes || {};
    this.contracts = contracts || {};
    this.account = account;
    this.targetPath = targetPath;
    this.transactionDetails = transactionDetails;
  }

  async declareClass(tag, { name, contract_path, casm_path }) {
    const contractPath =
      contract_path ||
      `${this.targetPath}/${this.name}_${name}.contract_class.json`;
    const casmPath =
      casm_path ||
      `${this.targetPath}/${this.name}_${name}.compiled_contract_class.json`;

    this.declarations[tag] = await declareContract(
      this.account,
      contractPath,
      casmPath
    );
    this.classes[tag] = {
      class_hash: this.declarations[tag].class_hash,
      abi: this.declarations[tag].abi,
    };
  }

  async declareAllClasses() {
    for (const [tag, { name, contract_path, casm_path }] of Object.entries(
      this.declare
    )) {
      await this.declareClass(tag, { name, contract_path, casm_path });
    }
  }

  async deployContract(contracts, transactionDetails) {
    const contractList = Array.isArray(contracts) ? contracts : [contracts];
    const payload = [];
    for (const data of contractList) {
      data.salt = data.salt || stark.randomAddress();
      const classData = (data.class && this.classes[data.class]) || {};
      data.class_hash = data.class_hash || classData.class_hash;
      payload.push({
        classHash: data.class_hash,
        salt: data.salt,
        unique: data.unique,
        constructorCalldata: data.calldata,
      });
    }

    const { contract_address: ContractAddresses, transaction_hash } =
      await account.deploy(payload, {
        ...this.transactionDetails,
        ...transactionDetails,
      });
    await account.waitForTransaction(transaction_hash);

    contractList.map((data, index) => {
      const contract_address = ContractAddresses[index];
      const tag = data.tag || contract_address;
      this.deployments[tag] = {
        ...data,
        contract_address,
        deployer_address: account.address,
        transaction_hash,
      };
      this.contracts[tag] = {
        contract_address,
        class_hash: data.class_hash,
        class: data.class,
      };
    });
  }

  async deployAllContracts() {
    const contractList = Object.entries(this.deploy).map(([tag, data]) => ({
      tag,
      ...data,
    }));
    await this.deployContract(contractList);
  }
}

const cmdArgs = commandLineArgs(cmdOptions);
const profile = cmdArgs.profile || "dev";
const directoryPath = cmdArgs.directory_path;

console.log(`Loading SAI profile: ${profile} from ${directoryPath}`);
const targetPath = resolvePath(`${directoryPath}/target/${profile}`);
const scarb_toml = loadToml(resolvePath(`${directoryPath}/Scarb.toml`));
const profile_toml = loadToml(
  resolvePath(`${directoryPath}/sai_${profile}.toml`)
);

// console.log(profile_toml);
const account = await loadAccount(profile_toml.account, cmdArgs);

const sai = loadSai(profile, account, targetPath, scarb_toml, profile_toml);

await sai.declareAllClasses();
console.log(sai.classes);

await sai.deployAllContracts();
console.log(sai.deployments);

// const declarations = await declareContracts(
//   account,
//   targetPath,
//   sai.name,
//   sai.contracts.declare
// );
// console.log(declarations);
// const deployments = await deployContracts(
//   account,
//   declarations,
//   sai.contracts.deploy
// );
// console.log(deployments);
// console.log(declaredContracts);
// const deployedContracts = await deployContracts(
//   account,
//   declaredContracts,
//   sai.contracts.deploy
// );

// const contract = deployedContracts.deployed_contract;
// console.log(deployedContracts);
