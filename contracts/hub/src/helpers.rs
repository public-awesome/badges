use std::fmt;

use cosmwasm_std::{Addr, Api, BlockInfo, Deps, Storage, Coin};
use sha2::{Digest, Sha256};

use badges::{Badge, MintRule};

use crate::error::ContractError;
use crate::state::{KEYS, OWNERS};

/// Each NFT's token id is simply the badge id and the serial separated by a pipe.
pub fn token_id(id: u64, serial: u64) -> String {
    format!("{}|{}", id, serial)
}

/// The message the user needs to sign to claim the badge under "by key" or "by keys" rule
pub fn message(id: u64, user: impl fmt::Display) -> String {
    format!("claim badge {} for user {}", id, user)
}

/// The hash function to be used to sign a message before signing it. Here we use SHA256.
/// https://docs.rs/sha2/latest/sha2/#usage
pub fn hash(msg: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(msg.as_bytes());
    hasher.finalize().to_vec()
}

/// A helper function to help casting Option to String
pub fn stringify_option(opt: Option<impl fmt::Display>) -> String {
    opt.map_or_else(|| "undefined".to_string(), |value| value.to_string())
}

/// Casting Vec<Coin> to a string
pub fn stringify_funds(funds: &[Coin]) -> String {
    funds
        .iter()
        .map(|coin| coin.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// This is basically a wrapper of `api.secp256k1_verify`, but instead of taking raw bytes in the
/// form of `&[u8]`, it takes the pubkey and signature as hex-encoded strings, and the original
/// message before hashing.
pub fn assert_valid_signature(
    api: &dyn Api,
    pubkey: &str,
    message: &str,
    signature: &str,
) -> Result<(), ContractError> {
    let msg_hash_bytes = hash(message);
    let key_bytes = hex::decode(pubkey)?;
    let sig_bytes = hex::decode(signature)?;

    if api.secp256k1_verify(&msg_hash_bytes, &sig_bytes, &key_bytes)? {
        Ok(())
    } else {
        Err(ContractError::InvalidSignature)
    }
}

// Assert the badge is available to be minted.
// Throw an error if the mint deadline or the max supply has been reached.
pub fn assert_available(
    badge: &Badge,
    block: &BlockInfo,
    amount: u64,
) -> Result<(), ContractError> {
    if let Some(expiry) = badge.expiry {
        if block.time.seconds() > expiry {
            return Err(ContractError::Expired);
        }
    }

    if let Some(max_supply) = badge.max_supply {
        if badge.current_supply + amount > max_supply {
            return Err(ContractError::SoldOut);
        }
    }

    Ok(())
}

// Assert the badge it NOT available to be minted. Throw an error if it is available.
pub fn assert_unavailable(badge: &Badge, block: &BlockInfo) -> Result<(), ContractError> {
    match assert_available(badge, block, 1) {
        Ok(_) => Err(ContractError::Available),
        Err(_) => Ok(()),
    }
}

/// Assert that an account has not already minted a badge.
pub fn assert_eligible(store: &dyn Storage, id: u64, user: &str) -> Result<(), ContractError> {
    if !OWNERS.contains(store, (id, user)) {
        Ok(())
    } else {
        Err(ContractError::already_claimed(id, user))
    }
}

/// Assert that a badge indeed uses the "by minter" rule, and that the sender is the minter.
pub fn assert_can_mint_by_minter(badge: &Badge, sender: &Addr) -> Result<(), ContractError> {
    match &badge.rule {
        MintRule::ByMinter(minter) => {
            if minter != sender {
                Err(ContractError::NotMinter)
            } else {
                Ok(())
            }
        },
        rule => Err(ContractError::wrong_mint_rule("by_minter", rule)),
    }
}

/// Assert that a badge indeed uses the "by key" rule, and the signature was produced by signing the
/// correct message with the correct privkey.
pub fn assert_can_mint_by_key(
    api: &dyn Api,
    id: u64,
    badge: &Badge,
    owner: &str,
    signature: &str,
) -> Result<(), ContractError> {
    // the badge must use the "by key" minting rule
    let pubkey = match &badge.rule {
        MintRule::ByKey(key) => key,
        rule => return Err(ContractError::wrong_mint_rule("by_key", rule)),
    };

    // the signature must be valid
    let message = message(id, owner);
    assert_valid_signature(api, pubkey, &message, signature)?;

    Ok(())
}

/// Assert that a badge indeed uses the "by keys" rule, and that the signature was produced by
/// signing the correct message using a whitelisted privkey.
pub fn assert_can_mint_by_keys(
    deps: Deps,
    id: u64,
    badge: &Badge,
    owner: &str,
    pubkey: &str,
    signature: &str,
) -> Result<(), ContractError> {
    // the badge must use the "by keys" minting rule
    match &badge.rule {
        MintRule::ByKeys => (),
        rule => return Err(ContractError::wrong_mint_rule("by_keys", rule)),
    }

    // the key must be whitelisted
    if !KEYS.contains(deps.storage, (id, pubkey)) {
        return Err(ContractError::key_does_not_exist(id));
    }

    // the signature must be valid
    let message = message(id, owner);
    assert_valid_signature(deps.api, pubkey, &message, signature)?;

    Ok(())
}
