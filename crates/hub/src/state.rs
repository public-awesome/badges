use cosmwasm_std::Addr;
use cw_item_set::Set;
use cw_storage_plus::{Item, Map};

use badges::{Badge, FeeRate};

/// Address of the developer
pub const DEVELOPER: Item<Addr> = Item::new("owner");

/// Address of badge nft contract
pub const NFT: Item<Addr> = Item::new("nft");

/// The fee rate, in ustars per byte, charged for storing data on-chain
pub const FEE_RATE: Item<FeeRate> = Item::new("fee_rate");

/// Total number of badges
pub const BADGE_COUNT: Item<u64> = Item::new("badge_count");

/// Badges, indexed by ids
pub const BADGES: Map<u64, Badge> = Map::new("badges");

/// Pubkeys that are whitelisted to mint a badge
pub const KEYS: Set<(u64, &str)> = Set::new("keys");

/// User addresses that have already claimed a badge. If a composite key {badge_id, user_addr}
/// exists in the map, then this user has already claimed.
///
/// Note that we don't verify the addresses here. The verifification is done by the NFT contract.
pub const OWNERS: Set<(u64, &str)> = Set::new("claimed");
