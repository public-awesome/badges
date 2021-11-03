use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError,
    StdResult,
};
use cw721::{AllNftInfoResponse, ContractInfoResponse, Cw721Execute, Cw721Query, NftInfoResponse};
use cw721_base::{state::TokenInfo, ContractError, Cw721Contract, QueryMsg};

use terra_trophies::hub::helpers::query_trophy_metadata;
use terra_trophies::metadata::{Metadata, Trait};
use terra_trophies::nft::ExecuteMsg;

// we extend the default Cw721 contract
pub type Parent<'a> = Cw721Contract<'a, Vec<Trait>, Empty>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    let parent = Parent::default();
    let contract_info = ContractInfoResponse {
        name: "Terra Trophies".to_string(),
        symbol: "n/a".to_string(),
    };
    parent.contract_info.save(deps.storage, &contract_info)?;
    parent.minter.save(deps.storage, &info.sender)?; // sender is minter
    parent.token_count.save(deps.storage, &0)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let parent = Parent::default();
    match msg {
        // our custom mint command
        ExecuteMsg::Mint {
            trophy_id,
            start_serial,
            owners,
        } => execute_mint(deps, env, info, trophy_id, start_serial, owners),

        // for all other commands, we simply routes to the parent Cw721 contract
        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => parent.transfer_nft(deps, env, info, recipient, token_id),
        ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => parent.send_nft(deps, env, info, contract, token_id, msg),
        ExecuteMsg::Approve {
            spender,
            token_id,
            expires,
        } => parent.approve(deps, env, info, spender, token_id, expires),
        ExecuteMsg::Revoke {
            spender,
            token_id,
        } => parent.revoke(deps, env, info, spender, token_id),
        ExecuteMsg::ApproveAll {
            operator,
            expires,
        } => parent.approve_all(deps, env, info, operator, expires),
        ExecuteMsg::RevokeAll {
            operator,
        } => parent.revoke_all(deps, env, info, operator),
    }
}

pub fn execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    trophy_id: u64,
    start_serial: u64,
    owners: Vec<String>,
) -> Result<Response, ContractError> {
    let parent = Parent::default();

    let minter = parent.minter.load(deps.storage)?;
    if info.sender != minter {
        return Err(ContractError::Unauthorized {});
    }

    let mut token_count = parent.token_count.load(deps.storage)?;
    let start_token_id = token_count + 1;
    token_count += owners.len() as u64;
    parent.token_count.save(deps.storage, &token_count)?;

    for (idx, owner) in owners.iter().enumerate() {
        let token_id = start_token_id + idx as u64;
        let serial = start_serial + idx as u64;
        let traits = vec![
            Trait {
                display_type: None,
                trait_type: "trophy id".to_string(),
                value: trophy_id.to_string(),
            },
            Trait {
                display_type: None,
                trait_type: "serial".to_string(),
                value: serial.to_string(),
            },
        ];
        let token = TokenInfo {
            owner: deps.api.addr_validate(owner)?,
            approvals: vec![],
            token_uri: None,
            extension: traits,
        };
        parent.tokens.save(deps.storage, &token_id.to_string(), &token)?;
    }

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("minter", info.sender)
        .add_attribute("trophy_id", trophy_id.to_string())
        .add_attribute("start_serial", start_serial.to_string())
        .add_attribute("amount_minted", owners.len().to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let parent = Parent::default();
    match msg {
        // our custom query commands
        QueryMsg::NftInfo {
            token_id,
        } => to_binary(&query_nft_info(deps, token_id)?),
        QueryMsg::AllNftInfo {
            token_id,
            include_expired,
        } => to_binary(&query_all_nft_info(deps, env, token_id, include_expired.unwrap_or(false))?),

        // for all other commands, we simply routes to the parent Cw721 contract
        QueryMsg::ContractInfo {} => to_binary(&parent.contract_info(deps)?),
        QueryMsg::Minter {} => to_binary(&parent.minter(deps)?),
        QueryMsg::NumTokens {} => to_binary(&parent.num_tokens(deps)?),
        QueryMsg::OwnerOf {
            token_id,
            include_expired,
        } => to_binary(&parent.owner_of(deps, env, token_id, include_expired.unwrap_or(false))?),
        QueryMsg::ApprovedForAll {
            owner,
            include_expired,
            start_after,
            limit,
        } => to_binary(&parent.all_approvals(
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
        } => to_binary(&parent.tokens(deps, owner, start_after, limit)?),
        QueryMsg::AllTokens {
            start_after,
            limit,
        } => to_binary(&parent.all_tokens(deps, start_after, limit)?),
    }
}

pub fn query_nft_info(deps: Deps, token_id: String) -> StdResult<NftInfoResponse<Metadata>> {
    let parent = Parent::default();
    let minter = parent.minter.load(deps.storage)?;
    let token = parent.tokens.load(deps.storage, &token_id)?;

    let traits = token.extension;
    let trophy_id = traits
        .iter()
        .cloned()
        .find(|t| t.trait_type == "trophy id")
        .ok_or_else(|| StdError::generic_err("cannot find `trophy_id` trait"))?
        .value;
    let serial = traits
        .iter()
        .cloned()
        .find(|t| t.trait_type == "serial")
        .ok_or_else(|| StdError::generic_err("cannot find `serial` trait"))?
        .value;

    // retrieve metadata of the trophy from hub
    let mut metadata = query_trophy_metadata(&deps.querier, &minter, trophy_id.parse().unwrap())?;

    // if the trophy's name is `trophy_name`, and the token's serial number is 69, then the
    // token's full name is `trophy_name #69`
    metadata.name = metadata.name.map(|name| format!("{} #{}", name, serial));

    // insert trophy id and serial into metadata's traits
    metadata.attributes = if let Some(attrs) = metadata.attributes {
        Some([&traits[..], &attrs[..]].concat())
    } else {
        Some(traits)
    };

    Ok(NftInfoResponse {
        token_uri: None,
        extension: metadata,
    })
}

pub fn query_all_nft_info(
    deps: Deps,
    env: Env,
    token_id: String,
    include_expired: bool,
) -> StdResult<AllNftInfoResponse<Metadata>> {
    let parent = Parent::default();
    Ok(AllNftInfoResponse {
        access: parent.owner_of(deps, env, token_id.clone(), include_expired)?,
        info: query_nft_info(deps, token_id)?,
    })
}

// MIGRATE

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    Ok(Response::default()) // do nothing
}
