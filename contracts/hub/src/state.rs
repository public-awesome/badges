use cosmwasm_std::{Addr, Empty};
use cw_storage_plus::{Item, Map};

use badges::Badge;

/// Address of badge nft contract
pub const NFT: Item<Addr> = Item::new("nft");

/// Total number of badges
pub const BADGE_COUNT: Item<u64> = Item::new("badge_count");

/// Badges, indexed by ids
pub const BADGES: Map<u64, Badge<Addr>> = Map::new("badges");

/// Pubkeys that are whitelisted to mint a badge
pub const KEYS: Map<(u64, &str), Empty> = Map::new("keys");

/// User addresses that have already claimed a badge. If a composite key {badge_id, user_addr}
/// exists in the map, then this user has already claimed.
///
/// Note that we don't verify the addresses here. The verifification is done by the NFT contract.
pub const OWNERS: Map<(u64, &str), Empty> = Map::new("claimed");
