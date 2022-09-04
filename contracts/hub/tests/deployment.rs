use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, to_binary, Addr, Empty, Reply, SubMsg, SubMsgResponse, SubMsgResult, WasmMsg,
};
use prost::Message;
use sg721::CollectionInfo;

use badge_hub::contract;
use badge_hub::error::ContractError;
use badge_hub::state::{BADGE_COUNT, NFT};

#[test]
fn instantiating() {
    let mut deps = mock_dependencies();

    let collection_info = CollectionInfo {
        creator: "larry".to_string(),
        description: "this is a test".to_string(),
        image: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
        external_link: Some("https://larry.engineer/".to_string()),
        royalty_info: None,
    };

    let res = contract::init(
        deps.as_mut(),
        mock_env(),
        mock_info("larry", &[]),
        168,
        collection_info.clone(),
    )
    .unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::reply_on_success(
            WasmMsg::Instantiate {
                admin: Some("larry".to_string()),
                code_id: 168,
                msg: to_binary(&sg721::InstantiateMsg {
                    name: "Badges".to_string(),
                    symbol: "B".to_string(),
                    minter: MOCK_CONTRACT_ADDR.to_string(),
                    collection_info,
                })
                .unwrap(),
                funds: vec![],
                label: "badge-nft".to_string()
            },
            1
        )]
    );
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "badges/hub/init"),
            attr("contract_name", "crates.io:badge-hub"),
            attr("contract_version", env!("CARGO_PKG_VERSION"))
        ]
    );

    let badge_count = BADGE_COUNT.load(deps.as_ref().storage).unwrap();
    assert_eq!(badge_count, 0);

    let version = cw2::get_contract_version(deps.as_ref().storage).unwrap();
    assert_eq!(version.contract, "crates.io:badge-hub");
    assert_eq!(version.version, env!("CARGO_PKG_VERSION"));
}

#[test]
fn rejecting_invalid_reply_id() {
    let mut deps = mock_dependencies();

    let err = badge_hub::entry::reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: 69420,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: None,
            }),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidReplyId(69420));
}

#[derive(Clone, PartialEq, Message)]
struct MsgInstantiateContractResponse {
    #[prost(string, tag = "1")]
    pub contract_address: prost::alloc::string::String,
    #[prost(bytes, tag = "2")]
    pub data: prost::alloc::vec::Vec<u8>,
}

#[test]
fn running_init_hook() {
    let mut deps = mock_dependencies();

    let res = contract::init_hook(
        deps.as_mut(),
        Reply {
            id: 1,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: Some(
                    MsgInstantiateContractResponse {
                        contract_address: "nft".to_string(),
                        data: vec![],
                    }
                    .encode_to_vec()
                    .into(),
                ),
            }),
        },
    )
    .unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(WasmMsg::Execute {
            contract_addr: "nft".to_string(),
            msg: to_binary(&sg721::ExecuteMsg::<Option<Empty>>::_Ready {}).unwrap(),
            funds: vec![]
        })]
    );
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "badges/hub/init_hook"),
            attr("nft", "nft".to_string()),
        ]
    );

    let nft_addr = NFT.load(deps.as_ref().storage).unwrap();
    assert_eq!(nft_addr, Addr::unchecked("nft"));
}
