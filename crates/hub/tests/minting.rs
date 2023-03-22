use cosmwasm_std::testing::{mock_dependencies, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{attr, to_binary, Addr, Empty, OwnedDeps, StdResult, Storage, SubMsg, WasmMsg};
use k256::ecdsa::{SigningKey, VerifyingKey};
use cw721_base::MintMsg;
use sg_metadata::Metadata;

use badge_hub::error::ContractError;
use badge_hub::helpers::{message, token_id};
use badge_hub::state::*;
use badge_hub::{execute, query};
use badges::{Badge, MintRule};

mod utils;

/// Return the mock privkey, its corresponding pubkey, and the pubkey in hex encoding
fn mock_keys() -> (SigningKey, VerifyingKey, String) {
    let privkey = utils::mock_privkey();
    let pubkey = VerifyingKey::from(&privkey);
    let pubkey_str = hex::encode(pubkey.to_bytes());
    (privkey, pubkey, pubkey_str)
}

fn set_badge_supply(store: &mut dyn Storage, id: u64, current_supply: u64) {
    BADGES
        .update(store, id, |badge| {
            let mut badge = badge.unwrap();
            badge.current_supply = current_supply;
            StdResult::Ok(badge)
        })
        .unwrap();
}

fn setup_test() -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    let mut deps = mock_dependencies();

    NFT.save(deps.as_mut().storage, &Addr::unchecked("nft")).unwrap();

    let default_badge = Badge {
        manager: Addr::unchecked("larry"),
        metadata: Metadata::default(),
        transferrable: true,
        rule: MintRule::ByKeys,
        expiry: Some(12345),
        max_supply: Some(100),
        current_supply: 98,
    };

    let (_, _, pubkey_str) = mock_keys();

    BADGES
        .save(
            deps.as_mut().storage,
            1,
            &Badge {
                rule: MintRule::ByMinter("larry".to_string()),
                ..default_badge.clone()
            },
        )
        .unwrap();

    BADGES
        .save(
            deps.as_mut().storage,
            2,
            &Badge {
                rule: MintRule::ByKey(pubkey_str.clone()),
                ..default_badge.clone()
            },
        )
        .unwrap();

    BADGES
        .save(
            deps.as_mut().storage,
            3,
            &Badge {
                rule: MintRule::ByKeys,
                ..default_badge
            },
        )
        .unwrap();

    KEYS.insert(deps.as_mut().storage, (3, &pubkey_str)).unwrap();

    deps
}

#[test]
fn minting_by_minter() {
    let mut deps = setup_test();

    // wrong mint type
    {
        let err = execute::mint_by_minter(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            3,
            utils::btreeset(&["jake"]),
            Addr::unchecked("larry"),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::wrong_mint_rule("by_minter", &MintRule::ByKeys));
    }

    // non-minter cannot mint
    {
        let err = execute::mint_by_minter(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            1,
            utils::btreeset(&["jake"]),
            Addr::unchecked("jake"),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::NotMinter);
    }

    // cannot mint past max supply
    {
        let err = execute::mint_by_minter(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            1,
            utils::btreeset(&["jake", "pumpkin", "doge"]),
            Addr::unchecked("larry"),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::SoldOut);
    }

    // cannot mint after expiry
    {
        let err = execute::mint_by_minter(
            deps.as_mut(),
            utils::mock_env_at_timestamp(99999),
            1,
            utils::btreeset(&["jake", "pumpkin"]),
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
                        msg: to_binary(&sg721::ExecuteMsg::<_, Empty>::Mint(MintMsg::<Option<Empty>> {
                            token_id: token_id(1, serial),
                            owner: owner.to_string(),
                            token_uri: None,
                            extension: None,
                        }))
                        .unwrap(),
                        funds: vec![],
                    })
                })
                .collect::<Vec<_>>()
        };

        let res = execute::mint_by_minter(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            1,
            utils::btreeset(&["pumpkin", "jake"]),
            Addr::unchecked("larry"),
        )
        .unwrap();
        // NOTE: with btreemap, the elements are sorted alphabetically
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

    let privkey = utils::mock_privkey();
    let msg = message(2, "larry");
    let signature = utils::sign(&privkey, &msg);

    // wrong mint rule
    {
        let err = execute::mint_by_key(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            3,
            "larry".to_string(),
            signature.clone(),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::wrong_mint_rule("by_key", &MintRule::ByKeys));
    }

    // attempt to mint with correct privkey but false message
    {
        let false_msg = message(2, "jake");
        let signature = utils::sign(&privkey, &false_msg);

        let err = execute::mint_by_key(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            2,
            "larry".to_string(),
            signature,
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidSignature);
    }

    // attempt to mint with false key wtith correct message
    {
        let false_privkey = utils::random_privkey();
        let signature = utils::sign(&false_privkey, &msg);

        let err = execute::mint_by_key(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            2,
            "larry".to_string(),
            signature,
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidSignature);
    }

    // properly mint
    {
        let res = execute::mint_by_key(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            2,
            "larry".to_string(),
            signature.clone(),
        )
        .unwrap();
        assert_eq!(
            res.messages,
            vec![SubMsg::new(WasmMsg::Execute {
                contract_addr: "nft".to_string(),
                msg: to_binary(&sg721::ExecuteMsg::<_, Empty>::Mint(MintMsg::<Option<Empty>> {
                    token_id: "2|99".to_string(),
                    owner: "larry".to_string(),
                    token_uri: None,
                    extension: None,
                }))
                .unwrap(),
                funds: vec![],
            })],
        );
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "badges/hub/mint_by_key"),
                attr("id", "2"),
                attr("serial", "99"),
                attr("recipient", "larry"),
            ],
        );

        // current supply should have been updated
        let badge = BADGES.load(deps.as_ref().storage, 2).unwrap();
        assert_eq!(badge.current_supply, 99);

        // larry should be marked as already received
        let res = query::owner(deps.as_ref(), 2, "larry");
        assert!(res.claimed);
    }

    // attempt to mint to the same user
    {
        let err = execute::mint_by_key(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            2,
            "larry".to_string(),
            signature.clone(),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::already_claimed(2, "larry"));
    }

    // attempt to mint after expiry
    {
        let err = execute::mint_by_key(
            deps.as_mut(),
            utils::mock_env_at_timestamp(99999),
            2,
            "larry".to_string(),
            signature.clone(),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Expired);
    }

    // attempt to mint after max supply is reached
    {
        set_badge_supply(deps.as_mut().storage, 2, 100);

        let err = execute::mint_by_key(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            2,
            "larry".to_string(),
            signature,
        )
        .unwrap_err();
        assert_eq!(err, ContractError::SoldOut);
    }
}

#[test]
fn minting_by_keys() {
    let mut deps = setup_test();

    let (privkey, _, pubkey_str) = mock_keys();
    let msg = message(3, "larry");
    let signature = utils::sign(&privkey, &msg);

    // wrong mint rule
    {
        let err = execute::mint_by_key(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            1,
            "larry".to_string(),
            signature.clone(),
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::wrong_mint_rule("by_key", &MintRule::by_minter("larry")),
        );
    }

    // attempt to mint with a whitelisted privkey but with wrong message
    {
        let false_msg = message(3, "jake");
        let signature = utils::sign(&privkey, &false_msg);

        let err = execute::mint_by_keys(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            3,
            "larry".to_string(),
            pubkey_str.clone(),
            signature,
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidSignature);
    }

    // attempt to mint with the correct message but a non-whitelisted privkey
    {
        let false_privkey = utils::random_privkey();
        let false_pubkey = VerifyingKey::from(&false_privkey);
        let false_pubkey_str = hex::encode(false_pubkey.to_bytes());
        let signature = utils::sign(&false_privkey, &msg);

        let err = execute::mint_by_keys(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            3,
            "larry".to_string(),
            false_pubkey_str,
            signature,
        )
        .unwrap_err();
        assert_eq!(err, ContractError::key_does_not_exist(3));
    }

    // properly mint
    {
        let res = execute::mint_by_keys(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            3,
            "larry".to_string(),
            pubkey_str.clone(),
            signature.clone(),
        )
        .unwrap();
        assert_eq!(
            res.messages,
            vec![SubMsg::new(WasmMsg::Execute {
                contract_addr: "nft".to_string(),
                msg: to_binary(&sg721::ExecuteMsg::<_, Empty>::Mint(MintMsg::<Option<Empty>> {
                    token_id: "3|99".to_string(),
                    owner: "larry".to_string(),
                    token_uri: None,
                    extension: None,
                }))
                .unwrap(),
                funds: vec![],
            })],
        );
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "badges/hub/mint_by_keys"),
                attr("id", "3"),
                attr("serial", "99"),
                attr("recipient", "larry"),
            ],
        );

        // current supply should have been updated
        let badge = BADGES.load(deps.as_ref().storage, 3).unwrap();
        assert_eq!(badge.current_supply, 99);

        // larry should be marked as already received
        let res = query::owner(deps.as_ref(), 3, "larry");
        assert!(res.claimed);

        // the pubkey should be removed from the whitelist
        let res = query::key(deps.as_ref(), 3, &pubkey_str);
        assert!(!res.whitelisted);
    }

    // attempt to mint to using the same privkey again
    {
        let msg = message(3, "jake");
        let signature = utils::sign(&privkey, &msg);

        let err = execute::mint_by_keys(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            3,
            "jake".to_string(),
            pubkey_str.clone(),
            signature,
        )
        .unwrap_err();
        assert_eq!(err, ContractError::key_does_not_exist(3));
    }

    // attempt to mint to the same user again
    {
        KEYS.insert(deps.as_mut().storage, (3, "larry")).unwrap();

        let err = execute::mint_by_keys(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            3,
        "larry".to_string(),
            pubkey_str,
            signature.clone(),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::already_claimed(3, "larry"));
    }

    // attempt to mint after expiry
    {
        let err = execute::mint_by_key(
            deps.as_mut(),
            utils::mock_env_at_timestamp(99999),
            3,
            "larry".to_string(),
            signature.clone(),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Expired);
    }

    // attempt to mint after max supply is reached
    {
        set_badge_supply(deps.as_mut().storage, 3, 100);

        let err = execute::mint_by_key(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            3,
            "larry".to_string(),
            signature,
        )
        .unwrap_err();
        assert_eq!(err, ContractError::SoldOut);
    }
}
