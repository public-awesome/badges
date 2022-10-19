use cosmwasm_std::{Decimal, DepsMut, StdResult, Storage};
use cw_storage_plus::Item;
use sg_std::Response;

use badges::FeeRate;

use crate::state::{BADGES, FEE_RATE};

const LEGACY_FEE_PER_BYTE: Item<Decimal> = Item::new("fee_per_byte");

/// Date and time (GMT): Wednesday, November 30, 2022 11:59:59 PM
const NEW_BADGE_3_EXPIRY: u64 = 1669852799;

pub fn migrate(deps: DepsMut, fee_rate: FeeRate) -> StdResult<Response> {
    // set separate fee rates for metadata and keys
    update_fee_rate(deps.storage, &fee_rate)?;

    // extend the minting deadline for badge 3
    update_badge_3_expiry(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "badges/hub/migrate")
        .add_attribute("from_version", "1.0.0")
        .add_attribute("to_version", "1.1.0")
        .add_attribute("metadata_fee_rate", fee_rate.metadata.to_string())
        .add_attribute("key_fee_rate", fee_rate.key.to_string()))
}

fn update_fee_rate(store: &mut dyn Storage, fee_rate: &FeeRate) -> StdResult<()> {
    LEGACY_FEE_PER_BYTE.remove(store);
    FEE_RATE.save(store, fee_rate)
}

fn update_badge_3_expiry(store: &mut dyn Storage) -> StdResult<()> {
    BADGES.update(
        store,
        3,
        |opt| {
            let mut badge = opt.unwrap();
            badge.expiry = Some(NEW_BADGE_3_EXPIRY);
            Ok(badge)
        },
    )
    .map(|_| ())
}
