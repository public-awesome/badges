use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{attr, Addr, Decimal, DepsMut, Empty, OwnedDeps};
use sg_metadata::Metadata;
use sg_std::Response;

use badge_hub::error::ContractError;
use badge_hub::state::*;
use badge_hub::{execute, query};
use badges::{Badge, MintRule, FeeRate};

mod utils;

// two valid secp256k1 public keys for testing purpose
const KEY_1: &str = "026f476708bd8fcc8a58bae717ee6922cdefd7917492dbc1a4c2f96d22ba30e470";
const KEY_2: &str = "03858cd06aadf3e26b05bc3d5ceacae2fb1ea4027b2c63730e3de39abea255ee8c";

// a valid ed25519 pubkey, but not a valid secp256k1 pubkey
const INVALID_KEY: &str = "0000000b7373682d6564323535313900000020060892d88619ba6f56bc2ec5f1daec09529fbfc4f7a6723006f19e724c3deea5";

fn setup_test() -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    let mut deps = mock_dependencies();

    DEVELOPER.save(deps.as_mut().storage, &Addr::unchecked("larry")).unwrap();
    NFT.save(deps.as_mut().storage, &Addr::unchecked("nft")).unwrap();
    BADGE_COUNT.save(deps.as_mut().storage, &0).unwrap();

    // here we test the creation of badges without fees
    // fee-related logics are tested in a separate file
    FEE_RATE
        .save(
            deps.as_mut().storage,
            &FeeRate {
                metadata: Decimal::zero(),
                key: Decimal::zero(),
            },
        )
        .unwrap();

    deps
}

fn mock_badge() -> Badge {
    Badge {
        manager: Addr::unchecked("larry"),
        metadata: Metadata {
            name: Some("first-badge".to_string()),
            ..Default::default()
        },
        transferrable: true,
        rule: MintRule::ByKeys,
        expiry: Some(12345),
        max_supply: Some(100),
        current_supply: 0,
    }
}

fn create_badge(deps: DepsMut, badge: &Badge) -> Response {
    execute::create_badge(
        deps,
        utils::mock_env_at_timestamp(10000),
        mock_info("creator", &[]),
        badge.clone(),
    )
    .unwrap()
}

#[test]
fn creating_unavailable_badges() {
    let mut deps = setup_test();

    // cannot create a badge that's already expired
    {
        let err = execute::create_badge(
            deps.as_mut(),
            utils::mock_env_at_timestamp(99999),
            mock_info("creator", &[]),
            mock_badge(),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Expired);
    }

    // cannot create a badge that has zero max supply
    {
        let mut badge = mock_badge();
        badge.max_supply = Some(0);

        let err = execute::create_badge(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            mock_info("creator", &[]),
            badge,
        )
        .unwrap_err();
        assert_eq!(err, ContractError::SoldOut);
    }
}

#[test]
fn creating_badge() {
    let mut deps = setup_test();

    // create the first badge
    {
        let badge = Badge {
            manager: Addr::unchecked("larry"),
            metadata: Metadata {
                name: Some("first-badge".to_string()),
                ..Default::default()
            },
            transferrable: true,
            rule: MintRule::ByMinter("larry".to_string()),
            expiry: Some(12345),
            max_supply: Some(100),
            current_supply: 0,
        };

        let res = create_badge(deps.as_mut(), &badge);
        assert_eq!(res.messages, vec![]);
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "badges/hub/create_badge"),
                attr("id", "1"),
                attr("fee", "[]"),
            ]
        );

        let cfg = query::config(deps.as_ref()).unwrap();
        assert_eq!(cfg.badge_count, 1);

        let b = query::badge(deps.as_ref(), 1).unwrap();
        assert_eq!(b, (1, badge).into());
    }

    // create the second badge
    {
        let badge = Badge {
            manager: Addr::unchecked("jake"),
            metadata: Metadata {
                name: Some("second-badge".to_string()),
                ..Default::default()
            },
            transferrable: false,
            rule: MintRule::ByKeys,
            expiry: None,
            max_supply: None,
            current_supply: 0,
        };

        let res = create_badge(deps.as_mut(), &badge);
        assert_eq!(res.messages, vec![]);
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "badges/hub/create_badge"),
                attr("id", "2"),
                attr("fee", "[]"),
            ]
        );

        let cfg = query::config(deps.as_ref()).unwrap();
        assert_eq!(cfg.badge_count, 2);

        let b = query::badge(deps.as_ref(), 2).unwrap();
        assert_eq!(b, (2, badge).into());
    }
}

#[test]
fn editing_badge() {
    let mut deps = setup_test();

    let badge = mock_badge();
    create_badge(deps.as_mut(), &badge);

    // non-manager cannot edit
    {
        let err = execute::edit_badge(
            deps.as_mut(),
            mock_info("jake", &[]),
            1,
            Metadata::default(),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::NotManager);
    }

    // manager can edit
    {
        let res = execute::edit_badge(
            deps.as_mut(),
            mock_info(badge.manager.as_str(), &[]),
            1,
            Metadata::default(),
        )
        .unwrap();
        assert_eq!(res.messages, vec![]);
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "badges/hub/edit_badge"),
                attr("id", "1"),
                attr("fee", "[]"),
            ],
        );

        let b = query::badge(deps.as_ref(), 1).unwrap();
        assert_eq!(b.metadata, Metadata::default());
    }
}

#[test]
fn adding_keys() {
    let mut deps = setup_test();

    // badge 1 has mint rule "by keys"
    let mut badge = mock_badge();
    create_badge(deps.as_mut(), &badge);

    // badge 2 has mint rule "by minter"
    badge.rule = MintRule::ByMinter("pumpkin".to_string());
    create_badge(deps.as_mut(), &badge);

    // non-manager cannot add key
    {
        let err = execute::add_keys(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            mock_info("jake", &[]),
            1,
            utils::btreeset(&[KEY_1]),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::NotManager);
    }

    // cannot add key if the badge is not of "by keys" mint rule
    {
        let err = execute::add_keys(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            mock_info("larry", &[]),
            2,
            utils::btreeset(&[KEY_1]),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::wrong_mint_rule("by_keys", &badge.rule));
    }

    // cannot add key if the badge is no longer available
    {
        let err = execute::add_keys(
            deps.as_mut(),
            utils::mock_env_at_timestamp(99999),
            mock_info("larry", &[]),
            1,
            utils::btreeset(&[KEY_1]),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Expired);
    }

    // cannot add invalid hex-encoded strings
    {
        let err = execute::add_keys(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            mock_info("larry", &[]),
            1,
            utils::btreeset(&["ngmi"]),
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::FromHex(hex::FromHexError::InvalidHexCharacter {
                c: 'n',
                index: 0
            }),
        );
    }

    // manager properly adds keys
    {
        let res = execute::add_keys(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            mock_info("larry", &[]),
            1,
            utils::btreeset(&[KEY_1, KEY_2]),
        )
        .unwrap();
        assert_eq!(res.messages, vec![]);
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "badges/hub/add_keys"),
                attr("id", "1"),
                attr("fee", "[]"),
                attr("keys_added", "2"),
            ],
        );

        let res = query::keys(deps.as_ref(), 1, None, None).unwrap();
        assert_eq!(res.keys, vec![KEY_1.to_string(), KEY_2.to_string()]);
    }
}

#[test]
fn rejecting_invalid_keys() {
    let mut deps = setup_test();

    // cannot create a new badge with invalid key
    {
        let err = execute::create_badge(
            deps.as_mut(),
            mock_env(),
            mock_info("larry", &[]),
            Badge {
                manager: Addr::unchecked("larry"),
                metadata: Metadata::default(),
                transferrable: false,
                rule: MintRule::ByKey(INVALID_KEY.into()),
                expiry: None,
                max_supply: None,
                current_supply: 0,
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidPubkey);
    }

    // cannot add invalid keys to an existing badge
    {
        // first, properly create a badge with the "by keys" minting rule
        execute::create_badge(
            deps.as_mut(),
            mock_env(),
            mock_info("larry", &[]),
            Badge {
                manager: Addr::unchecked("larry"),
                metadata: Metadata::default(),
                transferrable: false,
                rule: MintRule::ByKeys,
                expiry: None,
                max_supply: None,
                current_supply: 0,
            },
        )
        .unwrap();

        let err = execute::add_keys(
            deps.as_mut(),
            mock_env(),
            mock_info("larry", &[]),
            1,
            utils::btreeset(&[KEY_1, KEY_2, INVALID_KEY]),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidPubkey);
    }
}
