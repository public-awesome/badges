use std::collections::BTreeSet;

use cosmwasm_std::Decimal;
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
    /// The fee rate charged for when creating or editing badges, quoted in ustars per byte
    pub fee_per_byte: Decimal,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub enum SudoMsg {
    /// Set the fee rate for creating or editing badges. Callable by L1 governance.
    SetFeeRate {
        fee_per_byte: Decimal,
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    /// Create a new badge with the specified mint rule and metadata
    CreateBadge {
        /// Manager is the account that can 1) change the badge's metadata, and 2) if using the "by
        /// keys" mint rule, whitelist pubkeys.
        ///
        /// TODO: Make mananger an optional parameter; setting it to None meaning no one can change
        /// the metadata. Also, allow transferring of manager power in the `edit_badge` method.
        ///
        /// NOTE: If using the "by keys" minting rule, manager cannot be None, because a manager is
        /// is needed to whitelist keys.
        manager: String,
        /// The badge's metadata, defined by the OpenSea standard
        metadata: Metadata,
        /// Whether this badge is transferrable
        transferrable: bool,
        /// The rule by which this badge is to be minted. There are three available rules; see the
        /// docs of `badges::MintRule` for details.
        rule: MintRule,
        /// A deadline only before which the badge can be minted.
        /// Setting this to None means there is no deadline.
        /// Can only be set once when creating the badge; cannot be changed later.
        expiry: Option<u64>,
        /// The maximum amount of badge that can be minted. Note, users burning minted badges does
        /// NOT free up slots for new badges to be minted.
        /// Setting this to None means there is no max supply.
        /// Can only be set once when creating the badge; cannot be changed later.
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
        /// NOTE: Use BTreeSet, because the order of items in a HashSet may not be deterministic.
        /// See: https://www.reddit.com/r/rust/comments/krgvcu/is_the_iteration_order_of_hashset_deterministic/
        keys: BTreeSet<String>,
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
    /// For a badge with the "by minter" mint rule, mint new badges to a set of owners.
    /// Can only be invoked by the designated minter.
    MintByMinter {
        id: u64,
        /// NOTE: User BTreeSet instead of HashSet, the same reason as discussed above
        owners: BTreeSet<String>,
    },
    /// For a badge with the "by key" mint rule, mint a badge to the specified owner.
    /// The caller must submit a signature to prove they have the minting key.
    MintByKey {
        id: u64,
        owner: String,
        signature: String,
    },
    /// For a badge with the "by keys" mint rule, mint a badge to the specified owner.
    /// The caller must submit a signature to prove they have one of the whitelisted minting keys.
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
    /// Address of the contract's developer
    pub developer: String,
    /// Address of the Badge NFT contract
    pub nft: String,
    /// The total number of badges
    pub badge_count: u64,
    /// The fee rate charged for when creating or editing badges, quoted in ustars per byte
    pub fee_per_byte: Decimal,
}
