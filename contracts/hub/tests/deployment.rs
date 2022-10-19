use cosmwasm_std::testing::{mock_dependencies};
use cosmwasm_std::{attr, Addr, Decimal, SubMsg, WasmMsg, to_binary};

use badges::FeeRate;

use badge_hub::error::ContractError;
use badge_hub::state::{BADGE_COUNT, NFT, DEVELOPER};
use badge_hub::execute;

#[test]
fn instantiating() {
    let mut deps = mock_dependencies();

    let res = execute::init(
        deps.as_mut(),
        Addr::unchecked("larry"),
        FeeRate {
            metadata: Decimal::from_ratio(10u128, 1u128),
            key: Decimal::from_ratio(2u128, 1u128),
        },
    )
    .unwrap();
    assert_eq!(res.messages, vec![]);
    assert_eq!(res.attributes, vec![attr("action", "badges/hub/init")]);

    let badge_count = BADGE_COUNT.load(deps.as_ref().storage).unwrap();
    assert_eq!(badge_count, 0);
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
