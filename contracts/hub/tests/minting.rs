use cosmwasm_std::testing::{mock_dependencies, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{attr, to_binary, Addr, Empty, OwnedDeps, SubMsg, Timestamp, WasmMsg};
use cw_utils::Expiration;
use k256::ecdsa::{VerifyingKey};
use sg721::MintMsg;
use sg_metadata::Metadata;

use badge_hub::contract;
use badge_hub::error::ContractError;
use badge_hub::helpers::{token_id, token_uri};
use badge_hub::state::*;
use badges::{Badge, MintRule};

mod utils;

fn setup_test() -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    let mut deps = mock_dependencies();

    NFT.save(deps.as_mut().storage, &Addr::unchecked("nft")).unwrap();

    let default_badge = Badge {
        id: 0,
        manager: Addr::unchecked("larry"),
        metadata: Metadata::default(),
        rule: MintRule::ByKeys,
        expiry: Some(Expiration::AtTime(Timestamp::from_seconds(12345))),
        max_supply: Some(100),
        current_supply: 99,
    };

    let privkey = utils::mock_privkey();
    let pubkey = VerifyingKey::from(&privkey);
    let pubkey_str = hex::encode(pubkey.to_bytes());

    BADGES
        .save(
            deps.as_mut().storage,
            1,
            &Badge {
                id: 1,
                rule: MintRule::ByMinter("larry".to_string()),
                current_supply: 98,
                ..default_badge.clone()
            },
        )
        .unwrap();

    BADGES
        .save(
            deps.as_mut().storage,
            2,
            &Badge {
                id: 2,
                rule: MintRule::ByKey(pubkey_str),
                ..default_badge.clone()
            },
        )
        .unwrap();

    BADGES
        .save(
            deps.as_mut().storage,
            3,
            &Badge {
                id: 3,
                rule: MintRule::ByKeys,
                ..default_badge
            },
        )
        .unwrap();

    KEYS.save(deps.as_mut().storage, (3, utils::MOCK_PRIVKEY), &Empty {}).unwrap();

    deps
}

#[test]
fn minting_by_minter() {
    let mut deps = setup_test();

    // non-minter cannot mint
    {
        let err = contract::mint_by_minter(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            1,
            utils::hashset(&["jake"]),
            Addr::unchecked("jake"),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::NotMinter);
    }

    // cannot mint past max supply
    {
        let err = contract::mint_by_minter(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            1,
            utils::hashset(&["jake", "pumpkin", "doge"]),
            Addr::unchecked("larry"),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::SoldOut);
    }

    // cannot mint after expiry
    {
        let err = contract::mint_by_minter(
            deps.as_mut(),
            utils::mock_env_at_timestamp(99999),
            1,
            utils::hashset(&["jake", "pumpkin"]),
            Addr::unchecked("larry"),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Expired);
    }

    // minter properly mints
    {
        let expected_msgs = |owners: &[&str]| {
            owners
                .iter()
                .enumerate()
                .map(|(idx, owner)| {
                    let serial = 98 + (idx + 1) as u64;
                    SubMsg::new(WasmMsg::Execute {
                        contract_addr: "nft".to_string(),
                        msg: to_binary(&sg721::ExecuteMsg::Mint(MintMsg::<Option<Empty>> {
                            token_id: token_id(1, serial),
                            owner: owner.to_string(),
                            token_uri: Some(token_uri(1, serial)),
                            extension: None,
                        }))
                        .unwrap(),
                        funds: vec![],
                    })
                })
                .collect::<Vec<_>>()
        };

        let res = contract::mint_by_minter(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            1,
            utils::hashset(&["jake", "pumpkin"]),
            Addr::unchecked("larry"),
        )
        .unwrap();
        assert_eq!(res.messages, expected_msgs(&["jake", "pumpkin"]));
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "badges/hub/mint_by_minter"),
                attr("id", "1"),
                attr("amount", "2"),
            ],
        );
    }
}

#[test]
fn minting_by_key() {
    let mut deps = setup_test();

    // attempt to mint with correct privkey but false message
    {}

    // attempt to mint with false key wtith correct message
    {}

    // attempt to mint after expiry
    {}

    // properly mint
    {}

    // attempt to mint to the same user
    {}

    // attempt to mint after expiry
    {}

    // attempt to mint after max supply is reached
    {}
}

#[test]
fn minting_by_keys() {
    let mut deps = setup_test();

    // attempt to mint with a whitelisted privkey but with wrong message
    {}

    // attempt to mint with the correct message but a non-whitelisted privkey
    {}

    // properly mint
    {}

    // attempt to mint to using the same privkey again
    {}

    // attempt to mint to the same user again
    {}

    // attempt to mint after expiry
    {}

    // attempt to mint after max supply is reached
    {}
}
