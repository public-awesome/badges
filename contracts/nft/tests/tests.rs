use std::any::type_name;
use std::marker::PhantomData;

use badge_nft::entry;
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::{Addr, Empty, OwnedDeps, StdError};
use cw721::{AllNftInfoResponse, Cw721Query};
use sg721::CollectionInfo;
use sg_metadata::{Metadata, Trait};

use badge_nft::contract::{parse_token_id, prepend_traits, NftContract};
use badge_nft::msg::Extension;
use badges::{Badge, MintRule};

mod mock_querier;

fn mock_metadata() -> Metadata {
    Metadata {
        image: Some("ipfs://hash".to_string()),
        description: Some("This is a test".to_string()),
        name: Some("Test Badge".to_string()),
        attributes: Some(vec![Trait {
            display_type: None,
            trait_type: "rarity".to_string(),
            value: "SSR".to_string(),
        }]),
        ..Default::default()
    }
}

fn setup_test() -> OwnedDeps<MockStorage, MockApi, mock_querier::CustomQuerier, Empty> {
    let mut deps = OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: mock_querier::CustomQuerier::default(),
        custom_query_type: PhantomData,
    };

    deps.querier.hub.set_badge(
        69,
        Badge {
            manager: Addr::unchecked("larry"),
            metadata: mock_metadata(),
            transferrable: true,
            rule: MintRule::ByKeys,
            expiry: None,
            max_supply: None,
            current_supply: 420,
        },
    );

    deps.querier.hub.set_badge(
        420,
        Badge {
            manager: Addr::unchecked("jake"),
            metadata: mock_metadata(),
            transferrable: false,
            rule: MintRule::ByKeys,
            expiry: None,
            max_supply: None,
            current_supply: 88888,
        },
    );

    let contract = NftContract::default();

    contract
        .instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("hub", &[]),
            sg721::InstantiateMsg {
                name: "Badges".to_string(),
                symbol: "B".to_string(),
                minter: "hub".to_string(),
                collection_info: CollectionInfo {
                    creator: "larry".to_string(),
                    description: "this is a test".to_string(),
                    image: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
                    external_link: Some("https://larry.engineer/".to_string()),
                    royalty_info: None,
                },
            },
        )
        .unwrap();

    contract
        .ready(deps.as_mut(), mock_env(), mock_info("hub", &[]))
        .unwrap();

    contract
        .mint(
            deps.as_mut(),
            mock_env(),
            mock_info("hub", &[]),
            sg721::MintMsg::<Extension> {
                token_id: "69|420".to_string(),
                owner: "jake".to_string(),
                token_uri: None,
                extension: None,
            },
        )
        .unwrap();

    contract
        .mint(
            deps.as_mut(),
            mock_env(),
            mock_info("hub", &[]),
            sg721::MintMsg::<Extension> {
                token_id: "420|69".to_string(),
            owner: "pumpkin".to_string(),
            token_uri: None,
            extension: None,
            },
        )
        .unwrap();

    deps
}

#[test]
fn parsing_token_id() {
    assert_eq!(
        parse_token_id("").unwrap_err(),
        StdError::generic_err("invalid token id ``: must be in the format {serial}|{id}"),
    );
    assert_eq!(
        parse_token_id("ngmi").unwrap_err(),
        StdError::generic_err("invalid token id `ngmi`: must be in the format {serial}|{id}"),
    );
    assert_eq!(
        parse_token_id("1|2|3").unwrap_err(),
        StdError::generic_err("invalid token id `1|2|3`: must be in the format {serial}|{id}"),
    );
    assert_eq!(
        parse_token_id("69|").unwrap_err(),
        StdError::parse_err(type_name::<u64>(), "cannot parse integer from empty string"),
    );
    assert_eq!(
        parse_token_id("69|hfsp").unwrap_err(),
        StdError::parse_err(type_name::<u64>(), "invalid digit found in string"),
    );
    assert_eq!(parse_token_id("69|420").unwrap(), (69, 420));
}

#[test]
fn prepending_traits() {
    let metadata = prepend_traits(mock_metadata(), 69, 420);
    assert_eq!(
        metadata.attributes.unwrap(),
        vec![
            Trait {
                display_type: None,
                trait_type: "id".to_string(),
                value: "69".to_string(),
            },
            Trait {
                display_type: None,
                trait_type: "serial".to_string(),
                value: "420".to_string(),
            },
            Trait {
                display_type: None,
                trait_type: "rarity".to_string(),
                value: "SSR".to_string(),
            },
        ]
    );
}

#[test]
fn instantiating() {
    let deps = setup_test();
    let contract = NftContract::default();

    let minter = contract.parent.minter(deps.as_ref()).unwrap();
    assert_eq!(minter.minter, "hub");

    let info = contract.parent.contract_info(deps.as_ref()).unwrap();
    assert_eq!(info.name, "Badges");
    assert_eq!(info.symbol, "B");

    let info = contract.query_collection_info(deps.as_ref()).unwrap();
    assert_eq!(info.creator, "larry");
    assert!(info.royalty_info.is_none());

    let owner = contract
        .parent
        .owner_of(deps.as_ref(), mock_env(), "69|420".to_string(), false)
        .unwrap();
    assert_eq!(owner.owner, "jake");
}

#[test]
fn rejecting_transfers() {
    let mut deps = setup_test();
    let contract = NftContract::default();

    // attempt to transfer a transferrable token, should work
    entry::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("jake", &[]),
        badge_nft::msg::ExecuteMsg::TransferNft {
            recipient: "pumpkin".to_string(),
            token_id: "69|420".to_string(),
        },
    )
    .unwrap();
    let owner = contract
        .parent
        .owner_of(deps.as_ref(), mock_env(), "69|420".to_string(), false)
        .unwrap();
    assert_eq!(owner.owner, "pumpkin");

    // attempt to transfer a untransferrable token, should fail
    let err = entry::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("pumpkin", &[]),
        badge_nft::msg::ExecuteMsg::TransferNft {
            recipient: "jake".to_string(),
            token_id: "420|69".to_string(),
        },
    )
    .unwrap_err();
    // sg721_base::ContractError does not implement Eq or PartialEq, so we can't directly compare
    // the error types here
    assert_eq!(err.to_string(), "Generic error: badge 420 is not transferrable");
}

#[test]
fn querying_nft_info() {
    let deps = setup_test();
    let contract = NftContract::default();

    let info = contract.nft_info(deps.as_ref(), "69|420").unwrap();
    assert_eq!(info.token_uri.unwrap(), "https://badges-api.larry.engineer/metadata?id=69&serial=420");
    assert_eq!(info.extension, prepend_traits(mock_metadata(), 69, 420));
}

#[test]
fn querying_all_nft_info() {
    let deps = setup_test();
    let contract = NftContract::default();

    let AllNftInfoResponse {
        access,
        info,
    } = contract.all_nft_info(deps.as_ref(), mock_env(), "69|420".to_string(), None).unwrap();
    assert_eq!(access.owner, "jake");
    assert_eq!(info.token_uri.unwrap(), "https://badges-api.larry.engineer/metadata?id=69&serial=420");
    assert_eq!(info.extension, prepend_traits(mock_metadata(), 69, 420));
}
