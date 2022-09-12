# Badges

Badges is an NFT protocol that allows anyone to permissionlessly create digital badges, as rewards for participants of events, or people who achieve certain accomplishments.

## Overview

### Contracts

The Badges project consists of two smart contracts:

- `badge-hub` is where users create, edit, or mint new badges
- `badge-nft` is the non-fungible token that implements the [SG-721](https://crates.io/crates/sg721) interface, compatible with the [Stargaze marketplace](https://app.stargaze.zone/)

### Minting

Creation of new badges is permissionless. There currently isn't a creation fee, but one can be added later.

Each badge defines its own minting rule. There are three such rules to be chosen from:

* `by_minter` There is a designated minter, which can either be a human, a multisig, or another contract implementing custom minting logics. The minter can mint any amount of the badge to any user.
* `by_key` When creating the badge, the creator generates a private-public key pair, and provides the contract with the pubkey. The creator should then distribute the privkey off-chain. Any person who receives the privkey can mint an instance of the badge by submitting the signature of [a specified message](https://github.com/st4k3h0us3/badges/blob/363ab86d19c699202c7801f2d349af924c0cefb0/contracts/hub/src/helpers.rs#L16-L19) signed by the privkey. The privkey can be used many times, whereas each user can only mint once.
* `by_keys` Similar to the previous rule, but there are multiple privkeys, each can only be used once. Similarly, each user can only mint once.

Each badge can also optionally have a minting deadline and a max supply.

### Tokens

Badges are each identified by an integer number. The first badge ever to be created gets id #1, the second #2, and so on.

Each badge can be minted multiple times; the resulting **instances** of the badge are each identified by a **serial** number. For a given badge, the first instance to ever be minted gets serial #1, the second #2, and so on.

That is, each non-fungible token is identified by two numbers, the badge id and the serial number. The CW-721 `token_id` is defined by joining the two with a pipe character: `{id}|{serial}`. For example, the 420th instance of badge #69 has a `token_id` of `69|420`.

### Metadata

The metadata of badges are stored on-chain. However, the approach used by [`cw721-metadata-onchain`](https://github.com/CosmWasm/cw-nfts/tree/main/contracts/cw721-metadata-onchain) is not suitable for our use case. The said contract stores a separate copy of the metadata for each `token_id`. As instances of the same badge all have the same metadata, this is a huge waste of on-chain space.

Instead, only a single copy of the metadata is stored at the Hub contract. When a user queries the `nft_info` method on the NFT contract by providing a `token_id`, the NFT contract in turn queries the Hub contract for the metadata, and returns it to the user. In this way, we significantly reduce the contract's storage footprint.

### Purging

The Hub contract implements two methods, `purge_keys` and `purge_owners`, which allows anyone to delete certain contract data once they are no longer needed. This reduces the blockchain's state size and the burden for node operators.

## Deployment

### stargaze-1

| Contract  | Address |
| --------- | ------- |
| Badge Hub | TBD     |
| Badge NFT | TBD     |

### elgafar-1

| Contract  | Address |
| --------- | ------- |
| Badge Hub | TBD     |
| Badge NFT | TBD     |

## License

Contents of this repository are open source under [GNU General Public License v3](./LICENSE) or later.
