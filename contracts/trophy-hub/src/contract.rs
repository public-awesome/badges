use cosmwasm_std::{
    entry_point, to_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Reply,
    Response, StdError, StdResult, SubMsg, SubMsgExecutionResponse, WasmMsg,
};

use terra_trophies::hub::{ContractInfoResponse, ExecuteMsg, InstantiateMsg, QueryMsg, TrophyInfo};
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
        ExecuteMsg::CreateTrophy(metadata) => execute_create_trophy(deps, env, info, metadata),
        ExecuteMsg::EditTrophy {
            trophy_id,
            metadata,
        } => execute_edit_trophy(deps, env, info, trophy_id, metadata),
        ExecuteMsg::MintTrophy {
            trophy_id,
            owners,
        } => execute_mint_trophy(deps, env, info, trophy_id, owners),
    }
}

pub fn execute_create_trophy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    metadata: Metadata,
) -> StdResult<Response> {
    let state = State::default();

    let trophy_count = state.trophy_count.load(deps.storage)? + 1;
    state.trophy_count.save(deps.storage, &trophy_count)?;

    let trophy = TrophyInfo {
        creator: info.sender,
        metadata,
        instance_count: 0,
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

pub fn execute_mint_trophy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    trophy_id: u64,
    owners: Vec<String>,
) -> StdResult<Response> {
    let state = State::default();
    let nft = state.nft.load(deps.storage)?;

    let mut trophy = state.trophies.load(deps.storage, trophy_id.into())?;
    if info.sender != trophy.creator {
        return Err(StdError::generic_err("caller is not creator"));
    }

    let start_serial = trophy.instance_count + 1;
    trophy.instance_count += owners.len() as u64;
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

// TESTS

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{from_binary, ContractResult, Event, OwnedDeps};
    use terra_trophies::testing::assert_generic_error_message;

    fn setup_test() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies(&[]);

        reply(
            deps.as_mut(),
            mock_env(),
            Reply {
                id: 0,
                result: ContractResult::Ok(SubMsgExecutionResponse {
                    events: vec![
                        Event::new("instantiate_contract").add_attribute("contract_address", "nft")
                    ],
                    data: None,
                }),
            },
        )
        .unwrap();

        deps
    }

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies(&[]);
        let info = mock_info("deployer", &[]);

        let res = instantiate(
            deps.as_mut(),
            mock_env(),
            info,
            InstantiateMsg {
                nft_code_id: 123,
            },
        )
        .unwrap();

        assert_eq!(res.messages.len(), 1);
        assert_eq!(
            res.messages[0],
            SubMsg::reply_on_success(
                WasmMsg::Instantiate {
                    admin: Some("deployer".to_string()),
                    code_id: 123,
                    msg: to_binary(&Empty {}).unwrap(),
                    funds: vec![],
                    label: "trophy-nft".to_string(),
                },
                0,
            )
        );
    }

    #[test]
    fn proper_init_hook() {
        let mut deps = mock_dependencies(&[]);
        let env = mock_env();

        let response = SubMsgExecutionResponse {
            events: vec![
                Event::new("instantiate_contract").add_attribute("contract_address", "nft")
            ],
            data: None,
        };
        let _reply = Reply {
            id: 0,
            result: ContractResult::Ok(response),
        };
        reply(deps.as_mut(), env.clone(), _reply).unwrap();

        let res_bin = query(deps.as_ref(), env, QueryMsg::ContractInfo {}).unwrap();
        let res: ContractInfoResponse = from_binary(&res_bin).unwrap();
        assert_eq!(
            res,
            ContractInfoResponse {
                nft: "nft".to_string(),
                trophy_count: 0
            }
        );
    }

    #[test]
    fn editing_trophy() {
        let mut deps = setup_test();
        let env = mock_env();

        // create a trophy
        let msg = ExecuteMsg::CreateTrophy(Metadata {
            image: Some("ipfs://hash-to-image-1".to_string()),
            image_data: None,
            external_url: None,
            description: Some("I am trophy number one".to_string()),
            name: Some("Trophy Number One".to_string()),
            attributes: None,
            background_color: None,
            animation_url: Some("ipfs://hash-of-video-1".to_string()),
            youtube_url: None,
        });
        execute(deps.as_mut(), env.clone(), mock_info("creator", &[]), msg).unwrap();

        // non-creator can't edit
        let metadata = Metadata {
            image: Some("ipfs://hash-to-image-2".to_string()),
            image_data: None,
            external_url: None,
            description: Some("I am trophy number two".to_string()),
            name: Some("Trophy Number Two".to_string()),
            attributes: None,
            background_color: None,
            animation_url: Some("ipfs://hash-of-video-2".to_string()),
            youtube_url: None,
        };
        let msg = ExecuteMsg::EditTrophy {
            trophy_id: 1,
            metadata,
        };
        let err = execute(deps.as_mut(), env.clone(), mock_info("non-creator", &[]), msg.clone());
        assert_generic_error_message(err, "caller is not creator");

        // creator can mint
        execute(deps.as_mut(), env.clone(), mock_info("creator", &[]), msg).unwrap();

        // metadata should have been updated
        let res: TrophyInfo<String> = from_binary(
            &query(
                deps.as_ref(),
                env,
                QueryMsg::TrophyInfo {
                    trophy_id: 1,
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(res.metadata.name, Some("Trophy Number Two".to_string()));
    }

    #[test]
    fn minting_trophy() {
        let mut deps = setup_test();
        let env = mock_env();

        let msg = ExecuteMsg::CreateTrophy(Metadata {
            image: Some("ipfs://hash-to-image-1".to_string()),
            image_data: None,
            external_url: None,
            description: Some("I am trophy number one".to_string()),
            name: Some("Trophy Number One".to_string()),
            attributes: None,
            background_color: None,
            animation_url: Some("ipfs://hash-of-video-1".to_string()),
            youtube_url: None,
        });
        let res = execute(deps.as_mut(), env.clone(), mock_info("creator", &[]), msg).unwrap();
        assert_eq!(res.attributes[1].value, "1"); // trophy id

        // non-creator can't mint
        let msg = ExecuteMsg::MintTrophy {
            trophy_id: 1,
            owners: vec!["alice".to_string(), "bob".to_string()],
        };
        let err = execute(deps.as_mut(), env.clone(), mock_info("non-minter", &[]), msg.clone());
        assert_generic_error_message(err, "caller is not creator");

        // creator can mint
        let res = execute(deps.as_mut(), env.clone(), mock_info("creator", &[]), msg).unwrap();
        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "nft".to_string(),
                msg: to_binary(&NftExecuteMsg::Mint {
                    trophy_id: 1,
                    start_serial: 1,
                    owners: vec!["alice".to_string(), "bob".to_string()]
                })
                .unwrap(),
                funds: vec![]
            })
        );

        // try mint a seconds time; should generate correct `start_serial`
        let msg = ExecuteMsg::MintTrophy {
            trophy_id: 1,
            owners: vec!["charlie".to_string()],
        };
        let res = execute(deps.as_mut(), env, mock_info("creator", &[]), msg).unwrap();
        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "nft".to_string(),
                msg: to_binary(&NftExecuteMsg::Mint {
                    trophy_id: 1,
                    start_serial: 3,
                    owners: vec!["charlie".to_string()]
                })
                .unwrap(),
                funds: vec![]
            })
        );
    }
}
