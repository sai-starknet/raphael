# Raphael - StarkNet Deployment Tool

A modern, robust deployment system for StarkNet smart contracts, similar to Dojo but focused specifically on contract deployment and management.

## 🚀 Features

- **Declarative Configuration**: Define your deployments in TOML files
- **Multi-Environment Support**: Separate configurations for dev, staging, production
- **State Management**: Track declarations and deployments with manifest files
- **Error Handling**: Comprehensive error reporting and validation
- **Type Safety**: Built with Rust for reliability and performance
- **StarkNet Integration**: Native support for StarkNet v0.14.0+

## 📁 Project Structure

```
raphael/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library exports
│   ├── sai.rs           # Core deployment logic
│   ├── config.rs        # Configuration management
│   ├── utils.rs         # StarkNet utilities
│   └── error.rs         # Error handling
├── example/
│   ├── sai_dev.toml     # Example configuration
│   ├── Scarb.toml       # Scarb project config
│   └── src/
│       └── contract.cairo # Example contract
├── Cargo.toml           # Rust dependencies
└── README.md           # This file
```

## 🛠️ Installation

### Prerequisites

- Rust 1.70+
- StarkNet account with sufficient ETH for gas
- Compiled Cairo contracts (Sierra + CASM files)

### Build from source

```bash
git clone https://github.com/sai-starknet/raphael.git
cd raphael
cargo build --release
```

## ⚙️ Configuration

### 1. Project Setup

Create a `Scarb.toml` file in your project root:

```toml
[package]
name = "my-starknet-project"
version = "0.1.0"

[dependencies]
starknet = ">=2.3.0"
```

### 2. Environment Configuration

Create a configuration file named `sai_{profile}.toml` (e.g., `sai_dev.toml`):

```toml
[account]
account_address = "0x123..."
private_key = "0xabc..."
rpc_url = "https://starknet-sepolia.public.blastapi.io"

[declare.my_contract]
name = "MyContract"

[deploy.my_deployment]
class = "my_contract"
constructor_calldata = ["0x1", "0x2"]
unique = false
```

### 3. Contract Compilation

Ensure your contracts are compiled and the artifacts are in `target/{profile}/`:

```bash
scarb build
```

This should generate:

- `{project}_{contract}.contract_class.json` (Sierra)
- `{project}_{contract}.compiled_contract_class.json` (CASM)

## 🚀 Usage

### Basic Deployment

```bash
# Deploy with dev profile
./target/release/raphael dev

# Deploy with custom profile
./target/release/raphael production

# Deploy from specific directory
./target/release/raphael dev -d ./my-project
```

### Advanced Options

```bash
# Only declare contracts (don't deploy)
./target/release/raphael dev --declare-only

# Only deploy contracts (skip declaration)
./target/release/raphael dev --deploy-only

# Verbose output
./target/release/raphael dev --verbose

# Override RPC URL
./target/release/raphael dev -u https://my-custom-rpc.com

# Override account address
./target/release/raphael dev -A 0x123...
```

## 📋 Configuration Reference

### Account Configuration

```toml
[account]
# Required: Your StarkNet account address
account_address = "0x..."

# Required: Authentication (choose one)
private_key = "0x..."           # Direct private key
# OR
keystore_path = "./key.json"    # Encrypted keystore file
password = "secret"             # Keystore password

# Required: StarkNet RPC endpoint
rpc_url = "https://..."

[network]
# Optional: Network settings
chain_id = "SN_SEPOLIA"
explorer_url = "https://sepolia.starkscan.co"
```

### Declaration Configuration

```toml
[declare.{tag}]
# Optional: Contract name (defaults to tag)
name = "MyContract"

# Optional: Custom file paths (auto-generated if not provided)
sierra_path = "./path/to/contract.json"
casm_path = "./path/to/compiled.json"
```

### Deployment Configuration

```toml
[deploy.{tag}]
# Required: Class reference (choose one)
class = "declared_contract_tag"  # Reference to declared contract
# OR
class_hash = "0x..."            # Direct class hash

# Optional: Deployment settings
salt = "0x123"                  # Custom salt (random if not provided)
unique = false                  # Unique per deployer address
constructor_calldata = ["0x1"]  # Constructor arguments
wait_for_acceptance = true      # Wait for transaction acceptance
```

## 📊 Manifest Files

Raphael generates manifest files at `target/{profile}/manifest_{profile}.json` containing:

```json
{
  "profile": "dev",
  "project_name": "my-project",
  "classes": {
    "my_contract": {
      "class_hash": "0x...",
      "declared_at": 12345
    }
  },
  "contracts": {
    "my_deployment": {
      "contract_address": "0x...",
      "class_hash": "0x...",
      "deployed_at": 12346
    }
  },
  "declarations": {
    /* declaration transaction details */
  },
  "deployments": {
    /* deployment transaction details */
  },
  "generated_at": 1640995200
}
```

## 🔍 Error Handling

Raphael provides detailed error messages for common issues:

- **Configuration Errors**: Missing required fields, invalid formats
- **Network Errors**: RPC connection issues, transaction failures
- **File Errors**: Missing contract artifacts, invalid JSON/TOML
- **StarkNet Errors**: Account issues, insufficient balance, contract errors

## 🛡️ Security

- **Private Keys**: Store private keys securely, prefer keystore files
- **Environment Variables**: Use environment variables for sensitive data
- **Network Selection**: Verify RPC URLs and network settings
- **Gas Estimation**: Monitor transaction costs

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [StarkNet](https://starknet.io) for the amazing L2 platform
- [Dojo](https://github.com/dojoengine/dojo) for inspiration on deployment tooling
- [Cairo](https://github.com/starkware-libs/cairo) for the smart contract language

## 📞 Support

- [GitHub Issues](https://github.com/sai-starknet/raphael/issues)
- [StarkNet Discord](https://discord.gg/starknet)
- [Documentation](https://docs.starknet.io)
