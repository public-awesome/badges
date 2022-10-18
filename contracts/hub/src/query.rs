use cosmwasm_std::{Deps, Order, StdResult};
use cw_storage_plus::Bound;

use badges::hub::{
    BadgeResponse, BadgesResponse, ConfigResponse, KeyResponse, KeysResponse, OwnerResponse,
    OwnersResponse,
};

use crate::state::*;

pub const CONTRACT_NAME: &str = "crates.io:badge-hub";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const DEFAULT_LIMIT: u32 = 10;
pub const MAX_LIMIT: u32 = 30;

pub fn config(deps: Deps) -> StdResult<ConfigResponse> {
    let developer_addr = DEVELOPER.load(deps.storage)?;
    let nft_addr = NFT.load(deps.storage)?;
    let badge_count = BADGE_COUNT.load(deps.storage)?;
    let fee_rate = FEE_RATE.load(deps.storage)?;
    Ok(ConfigResponse {
        developer: developer_addr.into(),
        nft: nft_addr.into(),
        badge_count,
        fee_rate,
    })
}

pub fn badge(deps: Deps, id: u64) -> StdResult<BadgeResponse> {
    let badge = BADGES.load(deps.storage, id)?;
    Ok((id, badge).into())
}

pub fn badges(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<BadgesResponse> {
    let start = start_after.map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let badges = BADGES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (id, badge) = item?;
            Ok((id, badge).into())
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(BadgesResponse {
        badges,
    })
}

pub fn key(deps: Deps, id: u64, pubkey: impl Into<String>) -> KeyResponse {
    let key = pubkey.into();
    let whitelisted = KEYS.contains(deps.storage, (id, &key));
    KeyResponse {
        key,
        whitelisted,
    }
}

pub fn keys(
    deps: Deps,
    id: u64,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<KeysResponse> {
    let start = start_after.map(|key| Bound::ExclusiveRaw(key.into_bytes()));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let keys = KEYS
        .prefix(id)
        .keys(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    Ok(KeysResponse {
        keys,
    })
}

/// This function takes `impl Into<String>` instead of `String` so that i can type a few characters
/// less in the unit tests =)
pub fn owner(deps: Deps, id: u64, user: impl Into<String>) -> OwnerResponse {
    let user = user.into();
    let claimed = OWNERS.contains(deps.storage, (id, &user));
    OwnerResponse {
        user,
        claimed,
    }
}

pub fn owners(
    deps: Deps,
    id: u64,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<OwnersResponse> {
    let start = start_after.map(|user| Bound::ExclusiveRaw(user.into_bytes()));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let owners = OWNERS
        .prefix(id)
        .keys(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    Ok(OwnersResponse {
        owners,
    })
}
