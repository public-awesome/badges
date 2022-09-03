#![allow(dead_code)]

use std::collections::BTreeSet;

use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Env, Timestamp};
use k256::ecdsa::{signature::Signer, Signature, SigningKey};
use rand::rngs::OsRng;

pub const MOCK_PRIVKEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

/// Return the private key based on the hex-encoded `MOCK_PRIVKEY`
pub fn mock_privkey() -> SigningKey {
    let privkey_bytes = hex::decode(MOCK_PRIVKEY).unwrap();
    SigningKey::from_bytes(&privkey_bytes).unwrap()
}

/// Generate a random private key
pub fn random_privkey() -> SigningKey {
    SigningKey::random(&mut OsRng)
}

/// Sign a message using the provided privkey, and encode the signature in hex
pub fn sign(privkey: &SigningKey, msg: &str) -> String {
    let sig: Signature = privkey.sign(msg.as_bytes());
    let sig_bytes = sig.to_vec();
    hex::encode(sig_bytes)
}

/// Cast a slice of strings into a btreeset
pub fn btreeset(slice: &[&str]) -> BTreeSet<String> {
    slice.iter().map(|s| s.to_string()).collect()
}

/// Return an `env` object at the specified UNIX timestamp
pub fn mock_env_at_timestamp(timestamp: u64) -> Env {
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(timestamp);
    env
}
