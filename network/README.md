# Test network fixtures

This directory contains the ledger configuration and contract ABI
specifications used by the unit test suite (`cargo test`).

- `config.json`: chain id, node address, and deployed contract addresses
  used to build the test `LedgerClient`.
- `abis/*.json`: slim contract specifications (`contractName` + `abi` only)
  extracted from the compiled artifacts shipped in the
  [`indy-besu-contracts`](https://www.npmjs.com/package/indy-besu-contracts)
  npm package (`abi/contracts/**/<Name>.json`).

## Regenerating the ABIs

When the contracts change, refresh the fixtures from the npm package:

```sh
npm pack @maxandriani/indy-besu-contracts
tar xzf indy-besu-contracts-*.tgz
# For each contract, keep only `contractName` and `abi`:
python3 - <<'EOF'
import json, pathlib

contracts = [
    "did/EthereumExtDidRegistry.sol/EthereumExtDidRegistry",
    "did/IndyDidRegistry.sol/IndyDidRegistry",
    "anoncreds/SchemaRegistry.sol/SchemaRegistry",
    "anoncreds/CredentialDefinitionRegistry.sol/CredentialDefinitionRegistry",
    "anoncreds/RevocationRegistry.sol/RevocationRegistry",
    "auth/RoleControl.sol/RoleControl",
    "auth/AccountControl.sol/AccountControl",
    "upgrade/UpgradeControl.sol/UpgradeControl",
    "migration/LegacyMappingRegistry.sol/LegacyMappingRegistry",
    "network/ValidatorControl.sol/ValidatorControl",
]
for c in contracts:
    src = json.load(open(f"package/abi/contracts/{c}.json"))
    name = c.rsplit("/", 1)[-1]
    out = {"contractName": src["contractName"], "abi": src["abi"]}
    json.dump(out, open(f"network/abis/{name}.json", "w"), indent=2)
EOF
```
