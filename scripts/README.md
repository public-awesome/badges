# Scripts

This folder contains scripts for deploying or interacting with Badges contracts.

## How to Use

Install dependencies:

```bash
cd badges/scripts
npm install
npm install -g ts-node
```

Add a signing key:

```bash
ts-node 1_manage_keys.ts add <keyname> [--coin-type number] [--key-dir path]
```

On a testnet with permissionless contract deployment, upload contract codes:

```bash
ts-node 2_store_codes.ts --hub-wasm path --nft-wasm path \
  --network mainnet|testnet|localhost --key <keyname> [--key-dir path]
```

On mainnet, it is necessary to create a governance proposal for each code. Use the [`starsd`](https://github.com/public-awesome/stargaze) CLI to create proposals.

Instantiate contracts:

```bash
ts-node 4_instantiate.ts --hub-code-id number --nft-code-id number \
  --network mainnet|testnet|localhost --key <keyname> [--key-dir path]
```

Create a badge with the `by_key` minting rule:

```bash
# TODO
```

Create a badge with the `by_keys` minting rule:

```bash
# TODO
```
