use std::collections::BTreeSet;

use cosmwasm_std::{to_binary, Addr, DepsMut, Empty, Env, MessageInfo, StdResult, WasmMsg};
use cw721_base::msg::MintMsg;
use sg_metadata::Metadata;
use sg_std::Response;

use badges::{Badge, FeeRate, MintRule};

use crate::{
    error::ContractError,
    fee::handle_fee,
    helpers::*,
    query,
    state::*,
};

pub fn init(deps: DepsMut, developer: Addr, fee_rate: FeeRate) -> StdResult<Response> {
    DEVELOPER.save(deps.storage, &developer)?;
    BADGE_COUNT.save(deps.storage, &0)?;
    FEE_RATE.save(deps.storage, &fee_rate)?;

    Ok(Response::new()
        .add_attribute("action", "badges/hub/init"))
}

pub fn set_nft(deps: DepsMut, sender_addr: Addr, nft: &str) -> Result<Response, ContractError> {
    let developer_addr = DEVELOPER.load(deps.storage)?;

    if sender_addr != developer_addr {
        return Err(ContractError::NotDeveloper);
    }

    if NFT.may_load(deps.storage)?.is_some() {
        return Err(ContractError::DoubleInit);
    }

    let nft_addr = deps.api.addr_validate(nft)?;

    NFT.save(deps.storage, &nft_addr)?;

    Ok(Response::new()
        .add_attribute("action", "badges/hub/set_nft")
        .add_attribute("nft", nft))
}

pub fn set_fee_rate(deps: DepsMut, fee_rate: FeeRate) -> StdResult<Response> {
    FEE_RATE.save(deps.storage, &fee_rate)?;

    Ok(Response::new()
        .add_attribute("action", "badges/hub/set_fee_rate")
        .add_attribute("metadata_fee_rate", fee_rate.metadata.to_string())
        .add_attribute("key_fee_rate", fee_rate.key.to_string()))
}

pub fn create_badge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    badge: Badge,
) -> Result<Response, ContractError> {
    // the badge must not have already expired or have a max supply of zero
    assert_available(&badge, &env.block, 1)?;

    // ensure the creator has paid a sufficient fee
    let fee_rate = FEE_RATE.load(deps.storage)?;
    let res = handle_fee(
        deps.as_ref().storage,
        &info,
        None,
        Some(&badge),
        fee_rate.metadata,
    )?;

    // if the badge uses "by key" mint rule, the key must be a valid secp256k1
    // public key
    if let MintRule::ByKey(key) = &badge.rule {
        let bytes = hex::decode(key)?;
        assert_valid_secp256k1_pubkey(&bytes)?;
    }

    let id = BADGE_COUNT.update(deps.storage, |id| StdResult::Ok(id + 1))?;
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
    let fee_rate = FEE_RATE.load(deps.storage)?;
    let res = handle_fee(
        deps.as_ref().storage,
        &info,
        Some(&badge.metadata),
        &metadata,
        fee_rate.metadata,
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
    let fee_rate = FEE_RATE.load(deps.storage)?;
    let res = handle_fee(
        deps.as_ref().storage,
        &info,
        None,
        &keys,
        fee_rate.key,
    )?;

    // the minting deadline must not have been reached
    // the max supply must not have been reached
    assert_available(&badge, &env.block, 1)?;

    // save the keys
    keys.iter().try_for_each(|key| -> Result<_, ContractError> {
        // key must be a of valid hex encoding
        let bytes = hex::decode(key)?;

        // key must be a valid secp256k1 public key
        assert_valid_secp256k1_pubkey(&bytes)?;

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

    // can only purge keys once the badge becomes unavailable to be minted
    assert_unavailable(&badge, &env.block)?;

    // need to collect the keys into a Vec first before creating a new iterator to delete them
    // because of how Rust works
    let res = query::keys(deps.as_ref(), id, None, limit)?;
    for key in &res.keys {
        KEYS.remove(deps.storage, (id, key))?;
    };

    Ok(Response::new()
        .add_attribute("action", "badges/hub/purge_keys")
        .add_attribute("id", id.to_string())
        .add_attribute("keys_purged", res.keys.len().to_string()))
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
    let res = query::owners(deps.as_ref(), id, None, limit)?;
    for owner in &res.owners {
        OWNERS.remove(deps.storage, (id, owner))?;
    };

    Ok(Response::new()
        .add_attribute("action", "badges/hub/purge_owners")
        .add_attribute("id", id.to_string())
        .add_attribute("owners_purged", res.owners.len().to_string()))
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
                msg: to_binary(&sg721::ExecuteMsg::<_, Empty>::Mint(MintMsg::<Option<Empty>> {
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
    assert_can_mint_by_key(deps.api, id, &badge, &owner, &signature)?;

    badge.current_supply += 1;
    BADGES.save(deps.storage, id, &badge)?;

    OWNERS.insert(deps.storage, (id, &owner))?;

    Ok(Response::new()
        .add_message(WasmMsg::Execute {
            contract_addr: nft_addr.to_string(),
            msg: to_binary(&sg721::ExecuteMsg::<_, Empty>::Mint(MintMsg::<Option<Empty>> {
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
    assert_can_mint_by_keys(deps.as_ref(), id, &badge, &owner, &pubkey, &signature)?;

    badge.current_supply += 1;
    BADGES.save(deps.storage, id, &badge)?;

    KEYS.remove(deps.storage, (id, &pubkey))?;
    OWNERS.insert(deps.storage, (id, &owner))?;

    Ok(Response::new()
        .add_message(WasmMsg::Execute {
            contract_addr: nft_addr.to_string(),
            msg: to_binary(&sg721::ExecuteMsg::<_, Empty>::Mint(MintMsg::<Option<Empty>> {
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
