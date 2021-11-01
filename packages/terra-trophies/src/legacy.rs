use cosmwasm_std::Addr;
use cw721::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::hub::{MintRule, TrophyInfo};
use crate::metadata::Metadata;

/// Outdated trophy info struct used in v0.3.0. During migration, we upgrade this to the latest
/// trophy info format
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct LegacyTrophyInfo {
    /// Address of the collection's creator, who has the authority to edit the collection
    pub creator: Addr,
    /// The collection's metadata
    pub metadata: Metadata,
    /// The number of tris trophy's instances
    pub instance_count: u64,
}

impl LegacyTrophyInfo {
    pub fn upgrade(
        &self,
        rule: MintRule<Addr>,
        expiry: Option<Expiration>,
        max_supply: Option<u64>,
    ) -> TrophyInfo<Addr> {
        TrophyInfo {
            creator: self.creator.clone(),
            rule,
            metadata: self.metadata.clone(),
            expiry,
            max_supply,
            current_supply: self.instance_count,
        }
    }
}
