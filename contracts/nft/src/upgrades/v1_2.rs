use cosmwasm_std::{DepsMut, StdResult};
use sg_std::Response;

use crate::contract::{CONTRACT_NAME, CONTRACT_VERSION};

pub fn migrate(deps: DepsMut) -> StdResult<Response> {
    // previously, we forgot to set the contract name and version according to
    // cw2 spec. we do this here now
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new())
}
