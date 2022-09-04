use cosmwasm_std::Addr;
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg_metadata::Metadata;

use crate::MintRule;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Badge<T> {
    /// Identifier of the badge
    pub id: u64,
    /// Account who has the authority to edit the badge's info.
    /// Generic `T` is either `String` or `cosmwasm_std::Addr`.
    pub manager: T,
    /// The badge's metadata
    pub metadata: Metadata,
    /// The rule by which instances of this badge are to be minted
    pub rule: MintRule,
    /// The timestamp only before which the badge can be minted
    pub expiry: Option<Expiration>,
    /// The maximum number of badge instances can be minted
    pub max_supply: Option<u64>,
    /// The current number of this badge
    ///
    /// NOTE: We don't consider that users may burn NFTs. `max_supply` refers to the maximum number
    /// of tokens that can ever be minted. A user burning their tokens does not make room for new
    /// tokens to be minted.
    pub current_supply: u64,
}

impl From<Badge<Addr>> for Badge<String> {
    fn from(badge: Badge<Addr>) -> Self {
        Badge {
            id: badge.id,
            manager: badge.manager.to_string(),
            metadata: badge.metadata,
            rule: badge.rule,
            expiry: badge.expiry,
            max_supply: badge.max_supply,
            current_supply: badge.current_supply,
        }
    }
}
