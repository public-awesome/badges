use cosmwasm_std::{Decimal, DepsMut, StdResult};
use cw_storage_plus::Item;
use sg_std::Response;

use badges::FeeRate;

use crate::state::FEE_RATE;

const LEGACY_FEE_PER_BYTE: Item<Decimal> = Item::new("fee_per_byte");

pub fn migrate(deps: DepsMut, fee_rate: FeeRate) -> StdResult<Response> {
    update_fee_rate(deps, &fee_rate)?;

    Ok(Response::new()
        .add_attribute("action", "badges/hub/migrate")
        .add_attribute("from_version", "1.0.0")
        .add_attribute("to_version", "1.1.0")
        .add_attribute("metadata_fee_rate", fee_rate.metadata.to_string())
        .add_attribute("key_fee_rate", fee_rate.key.to_string()))
}

fn update_fee_rate(deps: DepsMut, fee_rate: &FeeRate) -> StdResult<()> {
    LEGACY_FEE_PER_BYTE.remove(deps.storage);
    FEE_RATE.save(deps.storage, fee_rate)
}
