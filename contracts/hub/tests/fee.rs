use std::collections::BTreeSet;

use cosmwasm_std::testing::{mock_dependencies, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Decimal, DepsMut, Empty, Event, OwnedDeps, SubMsg, Uint128,
};
use cw_utils::PaymentError;
use k256::ecdsa::VerifyingKey;
use sg1::FeeError;
use sg_metadata::{Metadata, Trait};
use sg_std::{create_fund_fairburn_pool_msg, Response, NATIVE_DENOM};

use badge_hub::error::ContractError;
use badge_hub::{execute, query};
use badge_hub::state::*;
use badges::{Badge, MintRule};

mod utils;

// 10 ustars per bytes
fn mock_fee_per_byte() -> Decimal {
    Decimal::from_ratio(10u128, 1u128)
}

fn setup_test() -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    let mut deps = mock_dependencies();

    DEVELOPER.save(deps.as_mut().storage, &Addr::unchecked("larry")).unwrap();
    NFT.save(deps.as_mut().storage, &Addr::unchecked("nft")).unwrap();
    BADGE_COUNT.save(deps.as_mut().storage, &0).unwrap();
    FEE_PER_BYTE.save(deps.as_mut().storage, &mock_fee_per_byte()).unwrap();

    deps
}

fn assert_correct_sg1_output(res: &Response, fee_amount: u128) {
    let dev_amount = fee_amount * 10 / 100;
    let burn_amount = fee_amount * 40 / 100;
    let dist_amount = fee_amount - dev_amount - burn_amount;

    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(BankMsg::Send {
                to_address: "larry".to_string(),
                amount: coins(dev_amount, NATIVE_DENOM),
            }),
            SubMsg::new(BankMsg::Burn {
                amount: coins(burn_amount, NATIVE_DENOM),
            }),
            SubMsg::new(create_fund_fairburn_pool_msg(coins(dist_amount, NATIVE_DENOM)))
        ]
    );
    assert_eq!(
        res.events,
        vec![Event::new("fair-burn")
            .add_attribute("dev", "larry")
            .add_attribute("dev_amount", dev_amount.to_string())
            .add_attribute("burn_amount", burn_amount.to_string())
            .add_attribute("dist_amount", dist_amount.to_string())]
    );
}

#[test]
fn badge_creation_fee() {
    let mut deps = setup_test();

    let mock_badge = Badge {
        manager: Addr::unchecked("manager"),
        metadata: Metadata::default(),
        transferrable: false,
        rule: MintRule::ByKeys,
        expiry: None,
        max_supply: None,
        current_supply: 0,
    };

    let mut create = |amount: u128, denom: &str| -> Result<Response, ContractError> {
        execute::create_badge(
            deps.as_mut(),
            utils::mock_env_at_timestamp(10000),
            mock_info("creator", &coins(amount, denom)),
            mock_badge.clone(),
        )
    };

    let bytes = to_binary(&mock_badge).unwrap();
    let fee_amount = (Uint128::from(bytes.len() as u128) * mock_fee_per_byte()).u128();

    // try create without sending a fee, should fail
    {
        let err = create(0, NATIVE_DENOM).unwrap_err();
        assert_eq!(err, FeeError::from(PaymentError::NoFunds {}).into());
    }

    // try create with less than sufficient amount, should fail
    {
        let insufficient_amount = fee_amount * 9 / 10;

        let err = create(insufficient_amount, NATIVE_DENOM).unwrap_err();
        assert_eq!(err, FeeError::InsufficientFee(fee_amount, insufficient_amount).into());
    }

    // try create with correct amount but wrong denom, should fail
    {
        let err = create(fee_amount, "doge").unwrap_err();
        assert_eq!(
            err,
            FeeError::from(PaymentError::MissingDenom(NATIVE_DENOM.to_string())).into()
        );
    }

    // try create with correct amount and denom, should succeed
    {
        let res = create(fee_amount, NATIVE_DENOM).unwrap();
        assert_correct_sg1_output(&res, fee_amount);
    }
}

#[test]
fn badge_editing_fee() {
    let mut deps = setup_test();

    let old_metadata = Metadata {
        name: Some("skyrim".to_string()),
        attributes: Some(vec![
            Trait {
                display_type: None,
                trait_type: "high_king".to_string(),
                value: "torygg".to_string(),
            },
            Trait {
                display_type: None,
                trait_type: "capital".to_string(),
                value: "solitude".to_string(),
            },
        ]),
        ..Default::default()
    };

    let new_metadata = Metadata {
        name: Some("skyrim".to_string()),
        attributes: Some(vec![
            Trait {
                display_type: None,
                trait_type: "high_king".to_string(),
                value: "ulfric_stormcloak".to_string(),
            },
            Trait {
                display_type: None,
                trait_type: "capital".to_string(),
                value: "windhelm".to_string(),
            },
        ]),
        ..Default::default()
    };

    let mock_badge = Badge {
        manager: Addr::unchecked("manager"),
        metadata: old_metadata.clone(),
        transferrable: false,
        rule: MintRule::ByKeys,
        expiry: None,
        max_supply: None,
        current_supply: 0,
    };

    BADGES.save(deps.as_mut().storage, 1, &mock_badge).unwrap();

    // can't use closure here due to borrowing
    fn edit(deps: DepsMut, metadata: &Metadata, amount: u128) -> Result<Response, ContractError> {
        execute::edit_badge(
            deps,
            mock_info("manager", &coins(amount, NATIVE_DENOM)),
            1,
            metadata.clone(),
        )
    }

    // if data size is smaller, no fee should be charged
    {
        let metadata = Metadata::default();

        let res = edit(deps.as_mut(), &metadata, 0).unwrap();
        assert_eq!(res.messages, vec![]);

        let badge = BADGES.load(deps.as_ref().storage, 1).unwrap();
        assert_eq!(badge.metadata, metadata);
    }

    // reset badge
    BADGES.save(deps.as_mut().storage, 1, &mock_badge).unwrap();

    // calculate the expected fee amount
    let old_bytes = to_binary(&old_metadata).unwrap().len() as u128;
    let new_bytes = to_binary(&new_metadata).unwrap().len() as u128;
    let fee_amount = (Uint128::new(new_bytes - old_bytes) * mock_fee_per_byte()).u128();

    // not sending sufficient fee, should fail
    {
        let insufficient_amount = fee_amount * 9 / 10;

        let err = edit(deps.as_mut(), &new_metadata, insufficient_amount).unwrap_err();
        assert_eq!(err, FeeError::InsufficientFee(fee_amount, insufficient_amount).into());
    }

    // send sufficient fee, should succeed
    {
        let res = edit(deps.as_mut(), &new_metadata, fee_amount).unwrap();
        assert_correct_sg1_output(&res, fee_amount);
    }
}

#[test]
fn key_adding_fee() {
    let mut deps = setup_test();

    let mock_badge = Badge {
        manager: Addr::unchecked("manager"),
        metadata: Metadata::default(),
        transferrable: false,
        rule: MintRule::ByKeys,
        expiry: None,
        max_supply: None,
        current_supply: 0,
    };

    BADGES.save(deps.as_mut().storage, 1, &mock_badge).unwrap();

    let mock_keys = (1..20)
        .map(|_| {
            let privkey = utils::random_privkey();
            let pubkey = VerifyingKey::from(&privkey);
            hex::encode(pubkey.to_bytes())
        })
        .collect::<Vec<_>>();

    let mock_keys_set = mock_keys
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();

    let bytes = to_binary(&mock_keys).unwrap().len() as u128;
    let fee_amount = (Uint128::new(bytes) * mock_fee_per_byte()).u128();

    fn add(
        deps: DepsMut,
        keys: &BTreeSet<String>,
        amount: u128,
    ) -> Result<Response, ContractError> {
        execute::add_keys(
            deps,
            utils::mock_env_at_timestamp(10000),
            mock_info("manager", &coins(amount, NATIVE_DENOM)),
            1,
            keys.clone(),
        )
    }

    // not sending sufficient fee
    {
        let insufficient_amount = fee_amount * 9 / 10;

        let err = add(deps.as_mut(), &mock_keys_set, insufficient_amount).unwrap_err();
        assert_eq!(err, FeeError::InsufficientFee(fee_amount, insufficient_amount).into());
    }

    // sending sufficient fee
    {
        let res = add(deps.as_mut(), &mock_keys_set, fee_amount).unwrap();
        assert_correct_sg1_output(&res, fee_amount);

        let res = query::key(deps.as_ref(), 1, &mock_keys[7]);
        assert!(res.whitelisted);
    }
}
