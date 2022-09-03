use std::fmt;

use cosmwasm_std::{Addr, Api, BlockInfo, Deps, Storage};
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
pub fn assert_available<T>(
    badge: &Badge<T>,
    block: &BlockInfo,
    amount: u64,
) -> Result<(), ContractError> {
    if let Some(expiry) = badge.expiry {
        if expiry.is_expired(block) {
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
pub fn assert_unavailable<T>(badge: &Badge<T>, block: &BlockInfo) -> Result<(), ContractError> {
    match assert_available(badge, block, 1) {
        Ok(_) => Err(ContractError::Available),
        Err(_) => Ok(()),
    }
}

/// Assert that an account has not already minted a badge.
pub fn assert_eligible(store: &dyn Storage, id: u64, user: &str) -> Result<(), ContractError> {
    if OWNERS.may_load(store, (id, user))?.is_none() {
        Ok(())
    } else {
        Err(ContractError::already_claimed(id, user))
    }
}

/// TODO: add docs
pub fn assert_can_mint_by_minter<T>(badge: &Badge<T>, sender: &Addr) -> Result<(), ContractError> {
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

/// TODO: add docs
pub fn assert_can_mint_by_key<T>(
    api: &dyn Api,
    badge: &Badge<T>,
    owner: &str,
    signature: &str,
) -> Result<(), ContractError> {
    // the badge must use the "by key" minting rule
    let pubkey = match &badge.rule {
        MintRule::ByKey(key) => key,
        rule => return Err(ContractError::wrong_mint_rule("by_key", rule)),
    };

    // the signature must be valid
    let message = message(badge.id, owner);
    assert_valid_signature(api, pubkey, &message, signature)?;

    Ok(())
}

/// TODO: add docs
/// NOTE: This function does NOT update the `keys` map
pub fn assert_can_mint_by_keys<T>(
    deps: Deps,
    badge: &Badge<T>,
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
    if KEYS.may_load(deps.storage, (badge.id, pubkey))?.is_none() {
        return Err(ContractError::key_does_not_exist(badge.id));
    }

    // the signature must be valid
    let message = message(badge.id, owner);
    assert_valid_signature(deps.api, pubkey, &message, signature)?;

    Ok(())
}
