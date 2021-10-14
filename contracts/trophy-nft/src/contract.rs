use std::str::FromStr;

use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Order, Pair, Response,
    StdError, StdResult,
};
use cw0::maybe_addr;
use cw721::{
    AllNftInfoResponse, ApprovedForAllResponse, ContractInfoResponse, Cw721ReceiveMsg, Expiration,
    NftInfoResponse, NumTokensResponse, OwnerOfResponse, TokensResponse,
};
use cw721_base::msg::{InstantiateMsg, MinterResponse, QueryMsg};
use cw_storage_plus::Bound;

use terra_trophies::nft::ExecuteMsg;

use crate::state::{BatchInfo, State, TokenId, TokenInfo};

use self::helpers::{try_transfer_nft, try_update_approvals};

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

// INIT

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State::default();

    let info = ContractInfoResponse {
        name: msg.name,
        symbol: msg.symbol,
    };
    state.contract_info.save(deps.storage, &info)?;

    let minter = deps.api.addr_validate(&msg.minter)?;
    state.minter.save(deps.storage, &minter)?;

    state.batch_count.save(deps.storage, &0)?;
    state.token_count.save(deps.storage, &0)?;

    Ok(Response::default())
}

// EXECUTE

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::CreateBatch {
            name,
            description,
            image,
            owners,
        } => execute_batch_mint(deps, env, info, name, description, image, owners),
        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => execute_transfer_nft(deps, env, info, recipient, token_id),
        ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => execute_send_nft(deps, env, info, contract, token_id, msg),
        ExecuteMsg::Approve {
            spender,
            token_id,
            expires,
        } => execute_approve(deps, env, info, spender, token_id, expires),
        ExecuteMsg::Revoke {
            spender,
            token_id,
        } => execute_revoke(deps, env, info, spender, token_id),
        ExecuteMsg::ApproveAll {
            operator,
            expires,
        } => execute_approve_all(deps, env, info, operator, expires),
        ExecuteMsg::RevokeAll {
            operator,
        } => execute_revoke_all(deps, env, info, operator),
    }
}

pub fn execute_batch_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: String,
    description: String,
    image: String,
    owners: Vec<String>,
) -> StdResult<Response> {
    let state = State::default();

    let minter = state.minter.load(deps.storage)?;
    if info.sender != minter {
        return Err(StdError::generic_err("caller is not minter"));
    }

    let batch_count = state.batch_count.load(deps.storage)? + 1;
    state.batch_count.save(deps.storage, &batch_count)?;

    let new_token_count = owners.len() as u64;
    let token_count = state.token_count.load(deps.storage)? + new_token_count;
    state.token_count.save(deps.storage, &token_count)?;

    let batch_id = batch_count;
    let batch = BatchInfo {
        name,
        description,
        image,
    };
    state.batches.save(deps.storage, batch_count.into(), &batch)?;

    for (idx, owner) in owners.iter().enumerate() {
        let token_id = TokenId(batch_id, (idx + 1) as u64);
        let token = TokenInfo {
            owner: deps.api.addr_validate(owner)?,
            approvals: vec![],
        };
        state.tokens.save(deps.storage, token_id.into(), &token)?;
    }

    Ok(Response::new()
        .add_attribute("action", "batch_mint")
        .add_attribute("minter", info.sender)
        .add_attribute("batch_id", batch_id.to_string())
        .add_attribute("new_token_count", new_token_count.to_string()))
}

pub fn execute_transfer_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id_str: String,
) -> StdResult<Response> {
    try_transfer_nft(deps, &env, &info, &recipient, &token_id_str)?;

    Ok(Response::new()
        .add_attribute("action", "transfer_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("token_id", token_id_str))
}

pub fn execute_send_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    token_id_str: String,
    msg: Binary,
) -> StdResult<Response> {
    // Transfer token
    try_transfer_nft(deps, &env, &info, &contract, &token_id_str)?;

    let send = Cw721ReceiveMsg {
        sender: info.sender.to_string(),
        token_id: token_id_str.clone(),
        msg,
    };

    // Send message
    Ok(Response::new()
        .add_message(send.into_cosmos_msg(contract.clone())?)
        .add_attribute("action", "send_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", contract)
        .add_attribute("token_id", token_id_str))
}

pub fn execute_approve(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id_str: String,
    expires: Option<Expiration>,
) -> StdResult<Response> {
    try_update_approvals(deps, &env, &info, &spender, &token_id_str, true, expires)?;

    Ok(Response::new()
        .add_attribute("action", "approve")
        .add_attribute("sender", info.sender)
        .add_attribute("spender", spender)
        .add_attribute("token_id", token_id_str))
}

pub fn execute_revoke(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id_str: String,
) -> StdResult<Response> {
    try_update_approvals(deps, &env, &info, &spender, &token_id_str, false, None)?;

    Ok(Response::new()
        .add_attribute("action", "revoke")
        .add_attribute("sender", info.sender)
        .add_attribute("spender", spender)
        .add_attribute("token_id", token_id_str))
}

pub fn execute_approve_all(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: String,
    expires: Option<Expiration>,
) -> StdResult<Response> {
    let state = State::default();

    // reject expired data as invalid
    let expires = expires.unwrap_or_default();
    if expires.is_expired(&env.block) {
        return Err(StdError::generic_err("approval has already expired"));
    }

    // set the operator
    let operator_addr = deps.api.addr_validate(&operator)?;
    state.operators.save(deps.storage, (&info.sender, &operator_addr), &expires)?;

    Ok(Response::new()
        .add_attribute("action", "approve_all")
        .add_attribute("sender", info.sender)
        .add_attribute("operator", operator))
}

pub fn execute_revoke_all(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: String,
) -> StdResult<Response> {
    let state = State::default();
    let operator_addr = deps.api.addr_validate(&operator)?;
    state.operators.remove(deps.storage, (&info.sender, &operator_addr));

    Ok(Response::new()
        .add_attribute("action", "revoke_all")
        .add_attribute("sender", info.sender)
        .add_attribute("operator", operator))
}

// QUERY

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractInfo {} => to_binary(&query_contract_info(deps)?),
        QueryMsg::Minter {} => to_binary(&query_minter(deps)?),
        QueryMsg::NumTokens {} => to_binary(&query_num_tokens(deps)?),
        QueryMsg::OwnerOf {
            token_id,
            include_expired,
        } => to_binary(&query_owner_of(deps, env, token_id, include_expired.unwrap_or(false))?),
        QueryMsg::NftInfo {
            token_id,
        } => to_binary(&query_nft_info(deps, token_id)?),
        QueryMsg::AllNftInfo {
            token_id,
            include_expired,
        } => to_binary(&query_all_nft_info(deps, env, token_id, include_expired.unwrap_or(false))?),
        QueryMsg::ApprovedForAll {
            owner,
            include_expired,
            start_after,
            limit,
        } => to_binary(&query_all_approvals(
            deps,
            env,
            owner,
            include_expired.unwrap_or(false),
            start_after,
            limit,
        )?),
        QueryMsg::Tokens {
            owner,
            start_after,
            limit,
        } => to_binary(&query_tokens(deps, owner, start_after, limit)?),
        QueryMsg::AllTokens {
            start_after,
            limit,
        } => to_binary(&query_all_tokens(deps, start_after, limit)?),
    }
}

pub fn query_contract_info(deps: Deps) -> StdResult<ContractInfoResponse> {
    let state = State::default();
    state.contract_info.load(deps.storage)
}

pub fn query_minter(deps: Deps) -> StdResult<MinterResponse> {
    let state = State::default();
    let minter_addr = state.minter.load(deps.storage)?;
    Ok(MinterResponse {
        minter: minter_addr.into(),
    })
}

pub fn query_num_tokens(deps: Deps) -> StdResult<NumTokensResponse> {
    let state = State::default();
    let count = state.token_count.load(deps.storage)?;
    Ok(NumTokensResponse {
        count,
    })
}

pub fn query_owner_of(
    deps: Deps,
    env: Env,
    token_id_str: String,
    include_expired: bool,
) -> StdResult<OwnerOfResponse> {
    let state = State::default();
    let token_id = TokenId::from_str(&token_id_str)?;
    let info = state.tokens.load(deps.storage, token_id.into())?;

    // remove expired approvals, then humanize
    let approvals = info
        .approvals
        .iter()
        .filter(|apr| include_expired || !apr.is_expired(&env.block))
        .map(|apr| apr.humanize())
        .collect();

    Ok(OwnerOfResponse {
        owner: info.owner.to_string(),
        approvals,
    })
}

pub fn query_nft_info(
    deps: Deps,
    token_id_str: String,
) -> StdResult<NftInfoResponse<Option<Empty>>> {
    let state = State::default();
    let token_id = TokenId::from_str(&token_id_str)?;
    let batch = state.batches.load(deps.storage, token_id.batch_id().into())?;
    Ok(NftInfoResponse {
        // If the batch's name is `batchName`, and the token's serial number is 69, then the token's
        // full name is `batchName #69`
        name: format!("{} #{}", batch.name, token_id.serial()),
        description: batch.description,
        image: Some(batch.image),
        extension: None,
    })
}

pub fn query_all_nft_info(
    deps: Deps,
    env: Env,
    token_id_str: String,
    include_expired: bool,
) -> StdResult<AllNftInfoResponse<Option<Empty>>> {
    // This implementation is slightly less gas-efficient (as we run TokenId::from_str twice) but
    // the code is much cleaner, so I'm ok with it
    Ok(AllNftInfoResponse {
        access: query_owner_of(deps, env, token_id_str.clone(), include_expired)?,
        info: query_nft_info(deps, token_id_str)?,
    })
}

pub fn query_all_approvals(
    deps: Deps,
    env: Env,
    owner: String,
    include_expired: bool,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ApprovedForAllResponse> {
    let state = State::default();
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_addr.map(|addr| Bound::exclusive(addr.as_ref()));

    let owner_addr = deps.api.addr_validate(&owner)?;
    let res: StdResult<Vec<_>> = state
        .operators
        .prefix(&owner_addr)
        .range(deps.storage, start, None, Order::Ascending)
        .filter(|r| include_expired || r.is_err() || !r.as_ref().unwrap().1.is_expired(&env.block))
        .take(limit)
        .map(parse_approval)
        .collect();

    Ok(ApprovedForAllResponse {
        operators: res?,
    })
}

fn parse_approval(item: StdResult<Pair<Expiration>>) -> StdResult<cw721::Approval> {
    item.and_then(|(k, expires)| {
        let spender = String::from_utf8(k)?;
        Ok(cw721::Approval {
            spender,
            expires,
        })
    })
}

pub fn query_tokens(
    deps: Deps,
    owner: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let state = State::default();
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let owner_addr = deps.api.addr_validate(&owner)?;
    let pks: Vec<_> = state
        .tokens
        .idx
        .owner
        .prefix(owner_addr)
        .keys(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();

    let res: Result<Vec<_>, _> = pks.iter().map(|v| String::from_utf8(v.to_vec())).collect();
    let tokens = res.map_err(StdError::invalid_utf8)?;

    Ok(TokensResponse {
        tokens,
    })
}

pub fn query_all_tokens(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let state = State::default();
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_addr.map(|addr| Bound::exclusive(addr.as_ref()));

    let tokens: StdResult<Vec<String>> = state
        .tokens
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(k, _)| String::from_utf8_lossy(&k).to_string()))
        .collect();

    Ok(TokensResponse {
        tokens: tokens?,
    })
}

// MIGRATE

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    Ok(Response::default()) // do nothing
}

// HELPERS

mod helpers {
    use super::*;
    use crate::state::Approval;

    pub fn try_transfer_nft(
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        recipient: &str,
        token_id_str: &str,
    ) -> StdResult<TokenInfo> {
        let state = State::default();
        let token_id = TokenId::from_str(token_id_str)?;
        let mut token = state.tokens.load(deps.storage, token_id.clone().into())?;

        // ensure we have permissions
        check_can_send(deps.as_ref(), env, info, &token)?;

        // set owner and remove existing approvals
        token.owner = deps.api.addr_validate(recipient)?;
        token.approvals = vec![];
        state.tokens.save(deps.storage, token_id.into(), &token)?;

        Ok(token)
    }

    /// returns true if the sender can transfer ownership of the token
    fn check_can_send(
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
        token: &TokenInfo,
    ) -> StdResult<()> {
        let state = State::default();

        // owner can send
        if token.owner == info.sender {
            return Ok(());
        }

        // any non-expired token approval can send
        if token
            .approvals
            .iter()
            .any(|apr| apr.spender == info.sender && !apr.is_expired(&env.block))
        {
            return Ok(());
        }

        // operator can send
        let op = state.operators.may_load(deps.storage, (&token.owner, &info.sender))?;
        if let Some(ex) = op {
            if !ex.is_expired(&env.block) {
                return Ok(());
            }
        }

        Err(StdError::generic_err("caller is not authorized to send"))
    }

    pub fn try_update_approvals(
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        spender: &str,
        token_id_str: &str,
        // if add == false, remove. if add == true, remove then set with this expiration
        add: bool,
        expires: Option<Expiration>,
    ) -> StdResult<()> {
        let state = State::default();
        let token_id = TokenId::from_str(token_id_str)?;
        let mut token = state.tokens.load(deps.storage, token_id.clone().into())?;

        // ensure we have permissions
        check_can_approve(deps.as_ref(), env, info, &token)?;

        // update the approval list, remove any approvals that are:
        // - for the same spender
        // - already expired
        let spender_addr = deps.api.addr_validate(spender)?;
        token.approvals = token
            .approvals
            .into_iter()
            .filter(|apr| apr.spender != spender_addr)
            .filter(|apr| !apr.expires.is_expired(&env.block))
            .collect();

        // only difference between approve and revoke
        if add {
            // reject expired data as invalid
            let expires = expires.unwrap_or_default();
            if expires.is_expired(&env.block) {
                return Err(StdError::generic_err("approval has already expired"));
            }
            let approval = Approval {
                spender: spender_addr,
                expires,
            };
            token.approvals.push(approval);
        }

        state.tokens.save(deps.storage, token_id.into(), &token)?;
        Ok(())
    }

    /// returns true if the sender can execute approve or reject on the contract
    fn check_can_approve(
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
        token: &TokenInfo,
    ) -> StdResult<()> {
        let state = State::default();

        // owner can approve
        if token.owner == info.sender {
            return Ok(());
        }

        // operator can approve
        let op = state.operators.may_load(deps.storage, (&token.owner, &info.sender))?;
        if let Some(ex) = op {
            if !ex.is_expired(&env.block) {
                return Ok(());
            }
        }

        Err(StdError::generic_err("caller is not authorized to send"))
    }
}

// TESTS

#[cfg(test)]
mod tests {
    // WIP
}
