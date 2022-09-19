use cosmwasm_std::testing::{mock_dependencies};
use cosmwasm_std::{attr, Addr, Decimal, SubMsg, WasmMsg, to_binary};

use badge_hub::error::ContractError;
use badge_hub::state::{BADGE_COUNT, NFT, DEVELOPER};
use badge_hub::execute;

#[test]
fn instantiating() {
    let mut deps = mock_dependencies();

    let res = execute::init(
        deps.as_mut(),
        Addr::unchecked("larry"),
        Decimal::from_ratio(10u128, 1u128),
    )
    .unwrap();
    assert_eq!(res.messages, vec![]);
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
fn setting_nft() {
    let mut deps = mock_dependencies();

    DEVELOPER.save(deps.as_mut().storage, &Addr::unchecked("larry")).unwrap();

    // non-developer cannot set nft
    {
        let err = execute::set_nft(deps.as_mut(), Addr::unchecked("jake"), "nft").unwrap_err();
        assert_eq!(err, ContractError::NotDeveloper);

        let opt = NFT.may_load(deps.as_ref().storage).unwrap();
        assert!(opt.is_none())
    }

    // developer sets nft
    {
        let res = execute::set_nft(deps.as_mut(), Addr::unchecked("larry"), "nft").unwrap();
        assert_eq!(
            res.messages,
            vec![SubMsg::new(WasmMsg::Execute {
                contract_addr: "nft".to_string(),
                msg: to_binary(&badges::nft::ExecuteMsg::_Ready {}).unwrap(),
                funds: vec![]
            })],
        );
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "badges/hub/set_nft"),
                attr("nft", "nft"),
            ],
        );

        let nft = NFT.load(deps.as_ref().storage).unwrap();
        assert_eq!(nft, Addr::unchecked("nft"));
    }

    // cannot set twice
    {
        let err = execute::set_nft(deps.as_mut(), Addr::unchecked("larry"), "nft").unwrap_err();
        assert_eq!(err, ContractError::DoubleInit);
    }
}
