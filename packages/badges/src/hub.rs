use std::collections::HashSet;

use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg721::{CollectionInfo, RoyaltyInfoResponse};
use sg_metadata::Metadata;

use crate::MintRule;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct InstantiateMsg {
    /// Code ID of the Badge NFT contract
    pub nft_code_id: u64,
    /// Collection description, per SG-721 specs
    pub nft_info: CollectionInfo<RoyaltyInfoResponse>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    /// Create a new badge with the specified mint rule and metadata
    CreateBadge {
        manager: String,
        metadata: Metadata,
        rule: MintRule,
        expiry: Option<Expiration>,
        max_supply: Option<u64>,
    },
    /// Edit the metadata of an existing badge; only the manager can call
    EditBadge {
        id: u64,
        metadata: Metadata,
    },
    /// For a badge that uses the "by keys" mint rule, invoke this method to whitelist pubkeys.
    /// Only callable by the manager before the minting deadline or max supply has been reached.
    AddKeys {
        id: u64,
        keys: HashSet<String>,
    },
    /// Once a badge has expired or sold out, the whitelisted keys are no longer needed. Invoke this
    /// method to purge these keys from storage in order to reduce the chain's state size.
    PurgeKeys {
        id: u64,
        limit: Option<u32>,
    },
    /// Once a badge has expired or sold out, the list of users who have claimed it is no longer
    /// needed. Invoke this method to purge these user addresses in order to reduce the chain's
    /// state size.
    PurgeOwners {
        id: u64,
        limit: Option<u32>,
    },
    /// Mint new instances of a specified trophy to a list of addresses. Called only if the trophy's
    /// minting rule is set to `ByOwner` and if caller is the owner
    MintByMinter {
        id: u64,
        owners: HashSet<String>,
    },
    /// Mint a new instance of the trophy by submitting a signature. The message should be the
    /// caller's address, and the private key is the one created for this trophy. Called only if the
    /// trophy's minting rule is set to `BySignature`
    MintByKey {
        id: u64,
        owner: String,
        signature: String,
    },
    /// TODO: add docs
    MintByKeys {
        id: u64,
        owner: String,
        pubkey: String,
        signature: String,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// The contract's configurations. Returns ConfigResponse
    Config {},
    /// Info about a badge. Returns Badge<String>
    Badge {
        id: u64,
    },
    /// Enumerate infos of all badges. Returns Vec<Badge<String>>
    Badges {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    /// Whether a pubkey can be used to mint a badge. Returns bool
    Key {
        id: u64,
        pubkey: String,
    },
    /// List all pubkeys that can be used to mint a badge. Returns Vec<String>
    Keys {
        id: u64,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Whether a user has claimed the specified badge. Returns bool
    Owner {
        id: u64,
        owner: String,
    },
    /// List a users that have claimed the specified badge. Returns Vec<String>
    Owners {
        id: u64,
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ConfigResponse {
    /// Address of the Badge NFT contract
    pub nft: String,
    /// The total number of badges
    pub badge_count: u64,
}
