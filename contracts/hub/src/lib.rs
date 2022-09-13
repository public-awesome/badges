pub mod contract;
pub mod error;
pub mod fee;
pub mod helpers;
pub mod state;

#[cfg(not(feature = "library"))]
pub mod entry {
    use cosmwasm_std::{
        entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, StdResult,
    };
    use sg_std::Response;

    use badges::hub::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};
    use badges::Badge;

    use crate::contract;
    use crate::error::ContractError;

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        contract::init(
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
            1 => contract::init_hook(deps, reply),
            id => Err(ContractError::InvalidReplyId(id)),
        }
    }

    #[entry_point]
    pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> StdResult<Response> {
        match msg {
            SudoMsg::SetFeeRate {
                fee_per_byte,
            } => contract::set_fee_rate(deps, fee_per_byte),
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
                contract::create_badge(deps, env, info, badge)
            },
            ExecuteMsg::EditBadge {
                id,
                metadata,
            } => contract::edit_badge(deps, info, id, metadata),
            ExecuteMsg::AddKeys {
                id,
                keys,
            } => contract::add_keys(deps, env, info, id, keys),
            ExecuteMsg::PurgeKeys {
                id,
                limit,
            } => contract::purge_keys(deps, env, id, limit),
            ExecuteMsg::PurgeOwners {
                id,
                limit,
            } => contract::purge_owners(deps, env, id, limit),
            ExecuteMsg::MintByMinter {
                id,
                owners,
            } => contract::mint_by_minter(deps, env, id, owners, info.sender),
            ExecuteMsg::MintByKey {
                id,
                owner,
                signature,
            } => contract::mint_by_key(deps, env, id, owner, signature),
            ExecuteMsg::MintByKeys {
                id,
                owner,
                pubkey,
                signature,
            } => contract::mint_by_keys(deps, env, id, owner, pubkey, signature),
        }
    }

    #[entry_point]
    pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::Config {} => to_binary(&contract::query_config(deps)?),
            QueryMsg::Badge {
                id,
            } => to_binary(&contract::query_badge(deps, id)?),
            QueryMsg::Badges {
                start_after,
                limit,
            } => to_binary(&contract::query_badges(deps, start_after, limit)?),
            QueryMsg::Key {
                id,
                pubkey,
            } => to_binary(&contract::query_key(deps, id, pubkey)),
            QueryMsg::Keys {
                id,
                start_after,
                limit,
            } => to_binary(&contract::query_keys(deps, id, start_after, limit)?),
            QueryMsg::Owner {
                id,
                owner,
            } => to_binary(&contract::query_owner(deps, id, owner)),
            QueryMsg::Owners {
                id,
                start_after,
                limit,
            } => to_binary(&contract::query_owners(deps, id, start_after, limit)?),
        }
    }
}
