use cosmwasm_std::{to_binary, MessageInfo, Storage, Uint128, Decimal};
use sg_std::Response;

use crate::error::ContractError;
use crate::state::DEVELOPER;

// TODO: add docs
pub fn handle_fee<T: serde::Serialize>(
    store: &dyn Storage,
    info: &MessageInfo,
    old_data: Option<T>,
    new_data: T,
    fee_per_byte: Decimal,
) -> Result<Response, ContractError> {
    // compute how much storage (in bytes) is taken
    let old_bytes = old_data
        .map(|data| to_binary(&data))
        .transpose()?
        .map(|bytes| bytes.len())
        .unwrap_or(0);
    let new_bytes = to_binary(&new_data)?.len();
    let bytes_diff = new_bytes.saturating_sub(old_bytes);

    // fee amount is bytes * fee rate (ustars per bytes)
    let fee = Uint128::new(bytes_diff as u128) * fee_per_byte;

    let mut res = Response::new();

    if !fee.is_zero() {
        let developer = DEVELOPER.load(store)?;
        sg1::checked_fair_burn(info, fee.u128(), Some(developer), &mut res)?;
    }

    Ok(res)
}
