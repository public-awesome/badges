use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MintRule {
    /// Badges can be minted by a designated minter account.
    ///
    /// The minter can either be a human doing the minting manually, or a smart contract that
    /// implements custom minting rules.
    ByMinter(String),

    /// Badges can be minted upon the the signature signed by a designated private key. Provide the
    /// associated public key in hex encoding.
    ///
    /// This key can be reused as many time as possible for minting, as long as the badge's deadline
    /// and max supply have not been reached.
    ByKey(String),

    /// Similar to the `ByKey` rule, but there are multiple pubkeys, each can only be used once.
    ///
    /// To add a pubkey, use the `add_key` execute method. Keys can only be added before the minting
    /// deadline and max supply haven't been reached.
    ///
    /// Once either the minting deadline or the max supply is reached, anyone can invoke the
    /// `clear_keys` method to remove unused keys from the contract storage, thereby reducing the
    /// size of the chain's state.
    ByKeys,
}

impl fmt::Display for MintRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            MintRule::ByMinter(minter) => format!("by_minter:{}", minter),
            MintRule::ByKey(pubkey) => format!("by_key:{}", pubkey),
            MintRule::ByKeys => "by_keys".to_string(),
        };
        write!(f, "{}", s)
    }
}

impl MintRule {
    pub fn by_minter(minter: impl Into<String>) -> Self {
        MintRule::ByMinter(minter.into())
    }

    pub fn by_key(key: impl Into<String>) -> Self {
        MintRule::ByKey(key.into())
    }
}
