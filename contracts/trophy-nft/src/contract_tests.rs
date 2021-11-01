use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::{from_binary, Deps, Empty, OwnedDeps};
use cw721::{
    AllNftInfoResponse, ContractInfoResponse, NumTokensResponse, OwnerOfResponse, TokensResponse,
};
use cw721_base::{ContractError, QueryMsg};

use serde::de::DeserializeOwned;

use terra_trophies::metadata::{Metadata, Trait};
use terra_trophies::nft::ExecuteMsg;
use terra_trophies::testing::CustomQuerier;

use crate::contract::{execute, instantiate, query};

// TESTS

#[test]
fn proper_instantiation() {
    let deps = setup_test();

    let res: ContractInfoResponse = query_helper(deps.as_ref(), QueryMsg::ContractInfo {});
    assert_eq!(
        res,
        ContractInfoResponse {
            name: "Terra Trophies".to_string(),
            symbol: "n/a".to_string()
        }
    );

    let res: NumTokensResponse = query_helper(deps.as_ref(), QueryMsg::NumTokens {});
    assert_eq!(res.count, 0);

    let res: TokensResponse = query_helper(
        deps.as_ref(),
        QueryMsg::AllTokens {
            start_after: None,
            limit: None,
        },
    );
    assert_eq!(res.tokens.len(), 0);
}

#[test]
fn minting_nfts() {
    let mut deps = setup_test();

    let msg = ExecuteMsg::Mint {
        trophy_id: 1,
        start_serial: 1,
        owners: vec!["alice".to_string(), "bob".to_string(), "charlie".to_string()],
    };

    // only hub can mint
    let info = mock_info("not_hub", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    assert_eq!(res, Err(ContractError::Unauthorized {}));

    // hub can mint
    let info = mock_info("hub", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // ensure num tokens increases
    let res: NumTokensResponse = query_helper(deps.as_ref(), QueryMsg::NumTokens {});
    assert_eq!(res.count, 3);

    // make sure nft info is correct
    let res: AllNftInfoResponse<Metadata> = query_helper(
        deps.as_ref(),
        QueryMsg::AllNftInfo {
            token_id: "2".to_string(),
            include_expired: None,
        },
    );
    assert_eq!(res.access.owner, "bob".to_string());
    assert_eq!(res.info.extension.name.unwrap(), "Trophy Number One #2".to_string());

    // make sure traits are correct
    let traits = vec![
        Trait {
            display_type: None,
            trait_type: "trophy id".to_string(),
            value: "1".to_string(),
        },
        Trait {
            display_type: None,
            trait_type: "serial".to_string(),
            value: "2".to_string(),
        },
    ];
    assert_eq!(res.info.extension.attributes.unwrap(), traits);

    // list the token ids
    let res: TokensResponse = query_helper(
        deps.as_ref(),
        QueryMsg::AllTokens {
            start_after: None,
            limit: None,
        },
    );
    assert_eq!(res.tokens, vec!["1".to_string(), "2".to_string(), "3".to_string()]);
}

#[test]
fn transferring_nft() {
    let mut deps = setup_test();

    // firstly, mint a trophy instance to alice
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("hub", &[]),
        ExecuteMsg::Mint {
            trophy_id: 1,
            start_serial: 1,
            owners: vec!["alice".to_string()],
        },
    )
    .unwrap();

    let msg = ExecuteMsg::TransferNft {
        token_id: "1".to_string(),
        recipient: "bob".to_string(),
    };

    // charlie can't transfer
    let info = mock_info("charlie", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    assert_eq!(res, Err(ContractError::Unauthorized {}));

    // alice can transfer
    let info = mock_info("alice", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res: OwnerOfResponse = query_helper(
        deps.as_ref(),
        QueryMsg::OwnerOf {
            token_id: "1".to_string(),
            include_expired: None,
        },
    );
    assert_eq!(res.owner, "bob".to_string());
}

#[test]
fn querying_nft_by_owner() {
    let mut deps = setup_test();

    // mint instances of trophy 1
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("hub", &[]),
        ExecuteMsg::Mint {
            trophy_id: 1,
            start_serial: 1,
            owners: vec!["alice".to_string()],
        },
    )
    .unwrap();

    // mint instances of trophy 2
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("hub", &[]),
        ExecuteMsg::Mint {
            trophy_id: 2,
            start_serial: 5,
            owners: vec!["bob".to_string(), "alice".to_string()],
        },
    )
    .unwrap();

    let res: TokensResponse = query_helper(
        deps.as_ref(),
        QueryMsg::Tokens {
            owner: "alice".to_string(),
            start_after: None,
            limit: None,
        },
    );
    assert_eq!(res.tokens, vec!["1".to_string(), "3".to_string()]);
}

// HELPERS

fn setup_test() -> OwnedDeps<MockStorage, MockApi, CustomQuerier> {
    // create deps object
    let mut deps = OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: CustomQuerier::default(),
    };

    // instantiate contract
    let info = mock_info("hub", &[]);
    instantiate(deps.as_mut(), mock_env(), info, Empty {}).unwrap();

    deps
}

fn query_helper<T: DeserializeOwned>(deps: Deps, msg: QueryMsg) -> T {
    from_binary(&query(deps, mock_env(), msg).unwrap()).unwrap()
}
