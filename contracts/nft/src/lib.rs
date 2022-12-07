pub mod contract;
pub mod state;
pub mod upgrades;

#[cfg(not(feature = "library"))]
pub mod entry {
    use cosmwasm_std::{
        entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, StdResult,
    };
    use sg721_base::ContractError;
    use sg_std::Response;

    use badges::nft::{ExecuteMsg, InstantiateMsg, QueryMsg};

    use crate::{contract::*, upgrades};

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        NftContract::default().instantiate(deps, env, info, msg)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        let tract = NftContract::default();
        // Transfers and approvals are only allowed if the badge is transferrable
        match &msg {
            ExecuteMsg::TransferNft {
                token_id,
                ..
            } => tract.assert_transferrable(deps.as_ref(), token_id)?,
            ExecuteMsg::SendNft {
                token_id,
                ..
            } => tract.assert_transferrable(deps.as_ref(), token_id)?,
            ExecuteMsg::Approve {
                token_id,
                ..
            } => tract.assert_transferrable(deps.as_ref(), token_id)?,
            _ => (),
        }
        tract.parent.execute(deps, env, info, msg)
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        let tract = NftContract::default();
        // We implement two custom query methods: `nft_info` and `all_nft_info`.
        // For all other queries, simply dispatch them to the parent.
        match msg {
            QueryMsg::NftInfo {
                token_id,
            } => to_binary(&tract.nft_info(deps, token_id)?),
            QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => to_binary(&tract.all_nft_info(deps, env, token_id, include_expired)?),
            _ => tract.parent.query(deps, env, msg),
        }
    }

    #[entry_point]
    pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
        upgrades::v1_2::migrate(deps)
    }
}
