use std::collections::BTreeSet;

use cosmwasm_std::{
    to_binary, Addr, Decimal, Deps, DepsMut, Empty, Env, MessageInfo, Order, Reply, StdResult,
    SubMsg, WasmMsg,
};
use cw_storage_plus::Bound;
use sg721::{CollectionInfo, MintMsg, RoyaltyInfoResponse};
use sg_metadata::Metadata;
use sg_std::Response;

use badges::hub::ConfigResponse;
use badges::{Badge, MintRule};

use crate::error::ContractError;
use crate::fee::handle_fee;
use crate::helpers::*;
use crate::state::*;

pub const CONTRACT_NAME: &str = "crates.io:badge-hub";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const DEFAULT_LIMIT: u32 = 10;
pub const MAX_LIMIT: u32 = 30;

pub fn init(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nft_code_id: u64,
    nft_info: CollectionInfo<RoyaltyInfoResponse>,
    fee_per_byte: Decimal,
) -> StdResult<Response> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    DEVELOPER.save(deps.storage, &info.sender)?;
    BADGE_COUNT.save(deps.storage, &0)?;
    FEE_PER_BYTE.save(deps.storage, &fee_per_byte)?;

    Ok(Response::new()
        .add_submessage(SubMsg::reply_on_success(
            WasmMsg::Instantiate {
                admin: Some(info.sender.to_string()),
                code_id: nft_code_id,
                msg: to_binary(&sg721::InstantiateMsg {
                    name: "Badges".to_string(),
                    symbol: "B".to_string(),
                    minter: env.contract.address.to_string(),
                    collection_info: nft_info,
                })?,
                funds: vec![],
                label: "badge-nft".to_string(),
            },
            1,
        ))
        .add_attribute("action", "badges/hub/init")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}

pub fn init_hook(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    let res = cw_utils::parse_reply_instantiate_data(reply)?;
    let nft_addr = deps.api.addr_validate(&res.contract_address)?;
    NFT.save(deps.storage, &nft_addr)?;

    Ok(Response::new()
        .add_message(WasmMsg::Execute {
            contract_addr: nft_addr.to_string(),
            msg: to_binary(&sg721::ExecuteMsg::<Option<Empty>>::_Ready {})?,
            funds: vec![],
        })
        .add_attribute("action", "badges/hub/init_hook")
        .add_attribute("nft", nft_addr.to_string()))
}

pub fn set_fee_rate(deps: DepsMut, fee_per_byte: Decimal) -> StdResult<Response> {
    FEE_PER_BYTE.save(deps.storage, &fee_per_byte)?;

    Ok(Response::new()
        .add_attribute("action", "badges/hub/set_fee_rate")
        .add_attribute("fee_per_byte", fee_per_byte.to_string()))
}

#[allow(clippy::too_many_arguments)]
pub fn create_badge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    manager: String,
    metadata: Metadata,
    transferrable: bool,
    rule: MintRule,
    expiry: Option<u64>,
    max_supply: Option<u64>,
) -> Result<Response, ContractError> {
    let id = BADGE_COUNT.update(deps.storage, |id| StdResult::Ok(id + 1))?;

    let badge = Badge {
        id,
        manager: deps.api.addr_validate(&manager)?,
        metadata,
        transferrable,
        rule,
        expiry,
        max_supply,
        current_supply: 0,
    };

    // ensure the creator has paid a sufficient fee
    let res = handle_fee(
        deps.as_ref().storage,
        &info,
        None,
        Some(&badge),
    )?;

    // the badge must not have already expired or have a max supply of zero
    assert_available(&badge, &env.block, 1)?;

    BADGES.save(deps.storage, id, &badge)?;

    Ok(res
        .add_attribute("action", "badges/hub/create_badge")
        .add_attribute("id", id.to_string())
        .add_attribute("fee", stringify_funds(&info.funds)))
}

pub fn edit_badge(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
    metadata: Metadata,
) -> Result<Response, ContractError> {
    let mut badge = BADGES.load(deps.storage, id)?;

    if info.sender != badge.manager {
        return Err(ContractError::NotManager);
    }

    // ensure the manager pays a sufficient fee
    let res = handle_fee(
        deps.as_ref().storage,
        &info,
        Some(&badge.metadata),
        &metadata,
    )?;

    badge.metadata = metadata;
    BADGES.save(deps.storage, id, &badge)?;

    Ok(res
        .add_attribute("action", "badges/hub/edit_badge")
        .add_attribute("id", id.to_string())
        .add_attribute("fee", stringify_funds(&info.funds)))
}

pub fn add_keys(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
    keys: BTreeSet<String>,
) -> Result<Response, ContractError> {
    let badge = BADGES.load(deps.storage, id)?;

    // only the badge's manager can add keys
    if info.sender != badge.manager {
        return Err(ContractError::NotManager);
    }

    // the badge must be of "by keys" minting rule
    match &badge.rule {
        MintRule::ByKeys => (),
        rule => return Err(ContractError::wrong_mint_rule("by_keys", rule)),
    }

    // ensure the manager pays a sufficient fee
    let res = handle_fee(
        deps.as_ref().storage,
        &info,
        None,
        &keys,
    )?;

    // the minting deadline must not have been reached
    // the max supply must not have been reached
    assert_available(&badge, &env.block, 1)?;

    // save the keys
    keys.iter().try_for_each(|key| -> Result<_, ContractError> {
        // key must be a of valid hex encoding
        hex::decode(key)?;

        // the key must not already exist
        if KEYS.insert(deps.storage, (id, key))? {
            Ok(())
        } else {
            Err(ContractError::key_exists(id, key))
        }
    })?;

    Ok(res
        .add_attribute("action", "badges/hub/add_keys")
        .add_attribute("id", id.to_string())
        .add_attribute("fee", stringify_funds(&info.funds))
        .add_attribute("keys_added", keys.len().to_string()))
}

pub fn purge_keys(
    deps: DepsMut,
    env: Env,
    id: u64,
    limit: Option<u32>,
) -> Result<Response, ContractError> {
    let badge = BADGES.load(deps.storage, id)?;

    // the badge must be of "by keys" minting rule
    match &badge.rule {
        MintRule::ByKeys => (),
        rule => return Err(ContractError::wrong_mint_rule("by_keys", rule)),
    }

    // can only purge keys once the badge becomes unavailable to be minted
    assert_unavailable(&badge, &env.block)?;

    // need to collect the keys into a Vec first before creating a new iterator to delete them
    // because of how Rust works
    let keys = query_keys(deps.as_ref(), id, None, limit)?;
    for key in &keys {
        KEYS.remove(deps.storage, (id, key));
    };

    Ok(Response::new()
        .add_attribute("action", "badges/hub/purge_keys")
        .add_attribute("id", id.to_string())
        .add_attribute("keys_purged", keys.len().to_string()))
}

pub fn purge_owners(
    deps: DepsMut,
    env: Env,
    id: u64,
    limit: Option<u32>,
) -> Result<Response, ContractError> {
    let badge = BADGES.load(deps.storage, id)?;

    // can only purge user data once the badge becomes unavailable to be minted
    assert_unavailable(&badge, &env.block)?;

    // need to collect the user addresses into a Vec first before creating a new iterator to delete
    // them because of how Rust works
    let owners = query_owners(deps.as_ref(), id, None, limit)?;
    for owner in &owners {
        OWNERS.remove(deps.storage, (id, owner));
    };

    Ok(Response::new()
        .add_attribute("action", "badges/hub/purge_owners")
        .add_attribute("id", id.to_string())
        .add_attribute("owners_purged", owners.len().to_string()))
}

pub fn mint_by_minter(
    deps: DepsMut,
    env: Env,
    id: u64,
    owners: BTreeSet<String>,
    sender: Addr,
) -> Result<Response, ContractError> {
    let nft_addr = NFT.load(deps.storage)?;
    let mut badge = BADGES.load(deps.storage, id)?;

    let amount = owners.len() as u64;
    let start_serial = badge.current_supply + 1;

    assert_available(&badge, &env.block, amount)?;
    assert_can_mint_by_minter(&badge, &sender)?;

    badge.current_supply += amount;
    BADGES.save(deps.storage, id, &badge)?;

    let msgs = owners
        .into_iter()
        .enumerate()
        .map(|(idx, owner)| -> StdResult<_> {
            let serial = start_serial + (idx as u64);
            Ok(WasmMsg::Execute {
                contract_addr: nft_addr.to_string(),
                msg: to_binary(&sg721::ExecuteMsg::Mint(MintMsg::<Option<Empty>> {
                    token_id: token_id(id, serial),
                    owner,
                    token_uri: None,
                    extension: None,
                }))?,
                funds: vec![],
            })
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "badges/hub/mint_by_minter")
        .add_attribute("id", id.to_string())
        .add_attribute("amount", amount.to_string()))
}

pub fn mint_by_key(
    deps: DepsMut,
    env: Env,
    id: u64,
    owner: String,
    signature: String,
) -> Result<Response, ContractError> {
    let nft_addr = NFT.load(deps.storage)?;
    let mut badge = BADGES.load(deps.storage, id)?;

    assert_available(&badge, &env.block, 1)?;
    assert_eligible(deps.storage, id, &owner)?;
    assert_can_mint_by_key(deps.api, &badge, &owner, &signature)?;

    badge.current_supply += 1;
    BADGES.save(deps.storage, id, &badge)?;

    OWNERS.insert(deps.storage, (id, &owner))?;

    Ok(Response::new()
        .add_message(WasmMsg::Execute {
            contract_addr: nft_addr.to_string(),
            msg: to_binary(&sg721::ExecuteMsg::Mint(MintMsg::<Option<Empty>> {
                token_id: token_id(id, badge.current_supply),
                // NOTE: it's possible to avoid cloning and save a liiiittle bit of gas here, simply
                // by moving this `add_message` after the one `add_attribute` that uses `owner`.
                // however this makes the code uglier so i don't want to do it.
                // Stargaze has free gas price anyways (for now at least)
                owner: owner.clone(),
                token_uri: None,
                extension: None,
            }))?,
            funds: vec![],
        })
        .add_attribute("action", "badges/hub/mint_by_key")
        .add_attribute("id", id.to_string())
        .add_attribute("serial", badge.current_supply.to_string())
        .add_attribute("recipient", owner))
}

pub fn mint_by_keys(
    deps: DepsMut,
    env: Env,
    id: u64,
    owner: String,
    pubkey: String,
    signature: String,
) -> Result<Response, ContractError> {
    let nft_addr = NFT.load(deps.storage)?;
    let mut badge = BADGES.load(deps.storage, id)?;

    assert_available(&badge, &env.block, 1)?;
    assert_eligible(deps.storage, id, &owner)?;
    assert_can_mint_by_keys(deps.as_ref(), &badge, &owner, &pubkey, &signature)?;

    badge.current_supply += 1;
    BADGES.save(deps.storage, id, &badge)?;

    KEYS.remove(deps.storage, (id, &pubkey));
    OWNERS.insert(deps.storage, (id, &owner))?;

    Ok(Response::new()
        .add_message(WasmMsg::Execute {
            contract_addr: nft_addr.to_string(),
            msg: to_binary(&sg721::ExecuteMsg::Mint(MintMsg::<Option<Empty>> {
                token_id: token_id(id, badge.current_supply),
                owner: owner.clone(),
                token_uri: None,
                extension: None,
            }))?,
            funds: vec![],
        })
        .add_attribute("action", "badges/hub/mint_by_keys")
        .add_attribute("id", id.to_string())
        .add_attribute("serial", badge.current_supply.to_string())
        .add_attribute("recipient", owner))
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let developer_addr = DEVELOPER.load(deps.storage)?;
    let nft_addr = NFT.load(deps.storage)?;
    let badge_count = BADGE_COUNT.load(deps.storage)?;
    let fee_per_byte = FEE_PER_BYTE.load(deps.storage)?;
    Ok(ConfigResponse {
        developer: developer_addr.into(),
        nft: nft_addr.into(),
        badge_count,
        fee_per_byte,
    })
}

pub fn query_badge(deps: Deps, id: u64) -> StdResult<Badge<String>> {
    let badge = BADGES.load(deps.storage, id)?;
    Ok(badge.into())
}

pub fn query_badges(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<Badge<String>>> {
    let start = start_after.map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    BADGES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, v) = item?;
            Ok(v.into())
        })
        .collect()
}

pub fn query_key(deps: Deps, id: u64, pubkey: impl Into<String>) -> bool {
    KEYS.contains(deps.storage, (id, &pubkey.into()))
}

pub fn query_keys(
    deps: Deps,
    id: u64,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<String>> {
    let start = start_after.map(|key| Bound::ExclusiveRaw(key.into_bytes()));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    KEYS
        .prefix(id)
        .keys(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect()
}

/// This function takes `impl Into<String>` instead of `String` so that i can type a few characters
/// less in the unit tests =)
pub fn query_owner(deps: Deps, id: u64, user: impl Into<String>) -> bool {
    OWNERS.contains(deps.storage, (id, &user.into()))
}

pub fn query_owners(
    deps: Deps,
    id: u64,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<String>> {
    let start = start_after.map(|user| Bound::ExclusiveRaw(user.into_bytes()));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    OWNERS
        .prefix(id)
        .keys(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect()
}
