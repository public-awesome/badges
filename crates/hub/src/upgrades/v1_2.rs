use cosmwasm_std::{DepsMut, StdResult, Storage};
use sg_std::Response;

use badges::MintRule;

use crate::{
    contract::{CONTRACT_NAME, CONTRACT_VERSION},
    state::BADGES,
};

const NEW_BADGE_17_KEY: &str = "036986114808be5b9f9009754014bdf5ae210cc17c93f4e1d010164be74b8653f4";

pub fn migrate(deps: DepsMut) -> StdResult<Response> {
    // correct the claim key of badge 17
    update_badge_17_key(deps.storage)?;

    // set the contract version to v1.2.0
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("action", "badges/hub/migrate")
        .add_attribute("from_version", "1.1.0")
        .add_attribute("to_version", "1.2.0"))
}

fn update_badge_17_key(store: &mut dyn Storage) -> StdResult<()> {
    BADGES.update(store, 17, |opt| -> StdResult<_> {
        let mut badge = opt.unwrap();
        badge.rule = MintRule::ByKey(NEW_BADGE_17_KEY.into());
        Ok(badge)
    })?;
    Ok(())
}
