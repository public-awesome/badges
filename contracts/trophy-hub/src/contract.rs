use cosmwasm_std::{
    entry_point, to_binary, Addr, Api, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo,
    Reply, Response, StdError, StdResult, SubMsg, SubMsgExecutionResponse, WasmMsg,
};
use cw721::Expiration;

use base64;
use sha2::{Digest, Sha256};

use terra_trophies::hub::{
    ContractInfoResponse, ExecuteMsg, InstantiateMsg, MintRule, QueryMsg, TrophyInfo,
};
use terra_trophies::metadata::Metadata;
use terra_trophies::nft::ExecuteMsg as NftExecuteMsg;

use crate::state::State;

// INIT

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::new().add_submessage(SubMsg::reply_on_success(
        WasmMsg::Instantiate {
            admin: Some(info.sender.to_string()),
            code_id: msg.nft_code_id,
            msg: to_binary(&Empty {})?,
            funds: vec![],
            label: "trophy-nft".to_string(),
        },
        0,
    )))
}

// REPLY

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> StdResult<Response> {
    match reply.id {
        0 => init_hook(deps, reply.result.unwrap()),
        id => Err(StdError::generic_err(format!("invalid reply id: {}", id))),
    }
}

pub fn init_hook(deps: DepsMut, response: SubMsgExecutionResponse) -> StdResult<Response> {
    let event = response
        .events
        .iter()
        .find(|event| event.ty == "instantiate_contract")
        .ok_or_else(|| StdError::generic_err("cannot find `instantiate_contract` event"))?;

    let nft_addr = event
        .attributes
        .iter()
        .cloned()
        .find(|attr| attr.key == "contract_address")
        .ok_or_else(|| StdError::generic_err("cannot find `contract_address` attribute"))?
        .value;

    let state = State::default();
    state.nft.save(deps.storage, &deps.api.addr_validate(&nft_addr)?)?;
    state.trophy_count.save(deps.storage, &0)?;

    Ok(Response::default())
}

// EXECUTE

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::CreateTrophy {
            rule,
            metadata,
            expiry,
            max_supply,
        } => execute_create_trophy(deps, env, info, rule, metadata, expiry, max_supply),
        ExecuteMsg::EditTrophy {
            trophy_id,
            metadata,
        } => execute_edit_trophy(deps, env, info, trophy_id, metadata),
        ExecuteMsg::MintByMinter {
            trophy_id,
            owners,
        } => execute_mint_by_minter(deps, env, info, trophy_id, owners),
        ExecuteMsg::MintBySignature {
            trophy_id,
            signature,
        } => execute_mint_by_signature(deps, env, info, trophy_id, signature),
    }
}

pub fn execute_create_trophy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    rule: MintRule<String>,
    metadata: Metadata,
    expiry: Option<Expiration>,
    max_supply: Option<u64>,
) -> StdResult<Response> {
    let state = State::default();

    let trophy_count = state.trophy_count.load(deps.storage)? + 1;
    state.trophy_count.save(deps.storage, &trophy_count)?;

    let trophy = TrophyInfo {
        creator: info.sender,
        rule: rule.check(deps.api)?,
        metadata,
        expiry,
        max_supply,
        current_supply: 0,
    };
    state.trophies.save(deps.storage, trophy_count.into(), &trophy)?;

    Ok(Response::new()
        .add_attribute("action", "create_trophy")
        .add_attribute("trophy_id", trophy_count.to_string()))
}

pub fn execute_edit_trophy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    trophy_id: u64,
    metadata: Metadata,
) -> StdResult<Response> {
    let state = State::default();

    let mut trophy = state.trophies.load(deps.storage, trophy_id.into())?;
    if info.sender != trophy.creator {
        return Err(StdError::generic_err("caller is not creator"));
    }

    trophy.metadata = metadata;
    state.trophies.save(deps.storage, trophy_id.into(), &trophy)?;

    Ok(Response::new()
        .add_attribute("action", "edit_trophy")
        .add_attribute("trophy_id", trophy_id.to_string()))
}

pub fn execute_mint_by_minter(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    trophy_id: u64,
    owners: Vec<String>,
) -> StdResult<Response> {
    let state = State::default();
    let mut trophy = state.trophies.load(deps.storage, trophy_id.into())?;

    match &trophy.rule {
        MintRule::ByMinter(minter) => {
            _verify_minter(&info.sender, &minter)?;
        }
        _ => {
            return Err(StdError::generic_err("minting rule is not `ByMinter`"));
        }
    }

    _mint(deps, env, state, &mut trophy, trophy_id, owners)
}

pub fn execute_mint_by_signature(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    trophy_id: u64,
    signature: String,
) -> StdResult<Response> {
    let state = State::default();
    let mut trophy = state.trophies.load(deps.storage, trophy_id.into())?;

    match &trophy.rule {
        MintRule::BySignature(pubkey) => {
            _verify_signature(deps.api, &info.sender.to_string(), &signature, pubkey)?;
        }
        _ => {
            return Err(StdError::generic_err("minting rule is not `BySignature`"));
        }
    }

    _mint(deps, env, state, &mut trophy, trophy_id, vec![info.sender.to_string()])
}

fn _verify_minter(caller: &Addr, minter: &Addr) -> StdResult<()> {
    if caller == minter {
        Ok(())
    } else {
        Err(StdError::generic_err("caller is not minter"))
    }
}

fn _verify_signature(
    api: &dyn Api,
    message: &String,
    signature: &String,
    pubkey: &String,
) -> StdResult<()> {
    let msg_bytes = Sha256::new().chain(message).finalize();
    let sig_bytes = base64::decode(signature)
        .map_err(|_| StdError::generic_err("[base64]: failed to decode signature"))?;
    let pk_bytes = base64::decode(pubkey)
        .map_err(|_| StdError::generic_err("[base64]: failed to decode pubkey"))?;

    if api.secp256k1_verify(&msg_bytes, &sig_bytes, &pk_bytes)? {
        Ok(())
    } else {
        Err(StdError::generic_err("signature verification failed"))
    }
}

fn _mint(
    deps: DepsMut,
    env: Env,
    state: State,
    trophy: &mut TrophyInfo<Addr>,
    trophy_id: u64,
    owners: Vec<String>,
) -> StdResult<Response> {
    let nft = state.nft.load(deps.storage)?;
    let new_supply = owners.len() as u64;
    let start_serial = trophy.current_supply + 1;

    // check minting time has not elapsed and max supply will not be exceeded after this mint
    if let Some(expiry) = trophy.expiry {
        if expiry.is_expired(&env.block) {
            return Err(StdError::generic_err("minting time has elapsed"));
        }
    }
    if let Some(max_supply) = trophy.max_supply {
        if trophy.current_supply + new_supply > max_supply {
            return Err(StdError::generic_err("max supply exceeded"));
        }
    }

    // each account can only receive the trophy once
    for owner in &owners {
        let owner_addr = deps.api.addr_validate(owner)?;
        let claimed = state
            .claimed
            .load(deps.storage, (&owner_addr, trophy_id.into()))
            .unwrap_or_else(|_| false);

        if claimed {
            return Err(StdError::generic_err(format!("already minted: {}", owner)));
        } else {
            state.claimed.save(deps.storage, (&owner_addr, trophy_id.into()), &true)?;
        }
    }

    trophy.current_supply += new_supply;
    state.trophies.save(deps.storage, trophy_id.into(), &trophy)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: nft.to_string(),
            msg: to_binary(&NftExecuteMsg::Mint {
                trophy_id,
                start_serial,
                owners,
            })?,
            funds: vec![],
        }))
        .add_attribute("action", "mint_trophy")
        .add_attribute("trophy_id", trophy_id.to_string()))
}

// QUERIES

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractInfo {} => to_binary(&query_contract_info(deps)?),
        QueryMsg::TrophyInfo {
            trophy_id,
        } => to_binary(&query_trophy_info(deps, trophy_id)?),
    }
}

pub fn query_contract_info(deps: Deps) -> StdResult<ContractInfoResponse> {
    let state = State::default();
    Ok(ContractInfoResponse {
        nft: state.nft.load(deps.storage)?.to_string(),
        trophy_count: state.trophy_count.load(deps.storage)?,
    })
}

pub fn query_trophy_info(deps: Deps, trophy_id: u64) -> StdResult<TrophyInfo<String>> {
    let state = State::default();
    let trophy = state.trophies.load(deps.storage, trophy_id.into())?;
    Ok(trophy.into())
}

// MIGRATE

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    Ok(Response::default()) // do nothing
}
