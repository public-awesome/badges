use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, StdResult,
};
use sg_std::Response;

use badges::hub::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};
use badges::Badge;

use crate::error::ContractError;
use crate::{execute, query};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    execute::init(
        deps,
        env,
        info,
        msg.nft_code_id,
        msg.nft_info,
        msg.fee_per_byte,
    )
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        1 => execute::init_hook(deps, reply),
        id => Err(ContractError::InvalidReplyId(id)),
    }
}

#[entry_point]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> StdResult<Response> {
    match msg {
        SudoMsg::SetFeeRate {
            fee_per_byte,
        } => execute::set_fee_rate(deps, fee_per_byte),
    }
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateBadge {
            manager,
            metadata,
            transferrable,
            rule,
            expiry,
            max_supply,
        } => {
            let badge = Badge {
                manager: deps.api.addr_validate(&manager)?,
                metadata,
                transferrable,
                rule,
                expiry,
                max_supply,
                current_supply: 0,
            };
            execute::create_badge(deps, env, info, badge)
        },
        ExecuteMsg::EditBadge {
            id,
            metadata,
        } => execute::edit_badge(deps, info, id, metadata),
        ExecuteMsg::AddKeys {
            id,
            keys,
        } => execute::add_keys(deps, env, info, id, keys),
        ExecuteMsg::PurgeKeys {
            id,
            limit,
        } => execute::purge_keys(deps, env, id, limit),
        ExecuteMsg::PurgeOwners {
            id,
            limit,
        } => execute::purge_owners(deps, env, id, limit),
        ExecuteMsg::MintByMinter {
            id,
            owners,
        } => execute::mint_by_minter(deps, env, id, owners, info.sender),
        ExecuteMsg::MintByKey {
            id,
            owner,
            signature,
        } => execute::mint_by_key(deps, env, id, owner, signature),
        ExecuteMsg::MintByKeys {
            id,
            owner,
            pubkey,
            signature,
        } => execute::mint_by_keys(deps, env, id, owner, pubkey, signature),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Badge {
            id,
        } => to_binary(&query::badge(deps, id)?),
        QueryMsg::Badges {
            start_after,
            limit,
        } => to_binary(&query::badges(deps, start_after, limit)?),
        QueryMsg::Key {
            id,
            pubkey,
        } => to_binary(&query::key(deps, id, pubkey)),
        QueryMsg::Keys {
            id,
            start_after,
            limit,
        } => to_binary(&query::keys(deps, id, start_after, limit)?),
        QueryMsg::Owner {
            id,
            user,
        } => to_binary(&query::owner(deps, id, user)),
        QueryMsg::Owners {
            id,
            start_after,
            limit,
        } => to_binary(&query::owners(deps, id, start_after, limit)?),
    }
}
