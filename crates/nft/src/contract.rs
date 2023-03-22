use std::any::type_name;
use std::str::FromStr;

use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, StdError, StdResult, Storage};
use cw721::Cw721Query;
use sg_metadata::{Metadata, Trait};
use sg_std::Response;

use badges::hub::BadgeResponse;
use badges::nft::{AllNftInfoResponse, Extension, InstantiateMsg, NftInfoResponse};

use crate::state::API_URL;

pub const CONTRACT_NAME: &str = "crates.io:badge-hub";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct NftContract<'a> {
    pub parent: sg721_base::Sg721Contract<'a, Extension>,
}

impl<'a> NftContract<'a> {
    pub fn instantiate(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, sg721_base::ContractError> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        API_URL.save(deps.storage, &msg.api_url)?;

        self.parent.instantiate(
            deps,
            env,
            info,
            sg721::InstantiateMsg {
                name: "Badges".to_string(),
                symbol: "B".to_string(),
                minter: msg.hub,
                collection_info: msg.collection_info,
            },
        )
    }

    /// Assert that the badge is transferrable
    pub fn assert_transferrable(&self, deps: Deps, token_id: impl ToString) -> StdResult<()> {
        let (id, _) = parse_token_id(&token_id.to_string())?;
        let badge = self.query_badge(deps, id)?;
        if badge.transferrable {
            Ok(())
        } else {
            Err(StdError::generic_err(format!("badge {} is not transferrable", id)))
        }
    }

    /// Overrides vanilla cw721's `nft_info` method
    pub fn nft_info(&self, deps: Deps, token_id: impl ToString) -> StdResult<NftInfoResponse> {
        let (id, serial) = parse_token_id(&token_id.to_string())?;
        let uri = uri(deps.storage, id, serial)?;
        let badge = self.query_badge(deps, id)?;
        Ok(NftInfoResponse {
            token_uri: Some(uri),
            extension: prepend_traits(badge.metadata, id, serial),
        })
    }

    /// Overrides vanilla cw721's `all_nft_info` method
    pub fn all_nft_info(
        &self,
        deps: Deps,
        env: Env,
        token_id: impl ToString,
        include_expired: Option<bool>,
    ) -> StdResult<AllNftInfoResponse> {
        let access = self.parent.parent.owner_of(
            deps,
            env,
            token_id.to_string(),
            include_expired.unwrap_or(false),
        )?;
        let info = self.nft_info(deps, token_id)?;
        Ok(AllNftInfoResponse {
            access,
            info,
        })
    }

    /// To save storage space, we save the badge's metadata at the Hub contract, instead of saving
    /// a separate copy in each token's extension. This function queries the Hub contract for the
    /// metadata of a given token id.
    fn query_badge(&self, deps: Deps, id: u64) -> StdResult<BadgeResponse> {
        let minter = self.parent.parent.minter(deps)?;
        deps.querier.query_wasm_smart(
            &minter.minter,
            &badges::hub::QueryMsg::Badge {
                id,
            },
        )
    }
}

/// URL of an API serving the metadata of the NFT.
///
/// A benefit of dynamically generating the URL instead of saving it in the contract storage is that
/// if I later want to update the URL, I only need to change this one function, instead of changing
/// every token's data.
pub fn uri(store: &dyn Storage, id: u64, serial: u64) -> StdResult<String> {
    let api_url = API_URL.load(store)?;
    Ok(format!("{}?id={}&serial={}", api_url, id, serial))
}

/// Split a token id into badge id and serial number.
/// The token id must be in the format `{u64}|{u64}`, where the 1st number is id and 2nd is serial.
pub fn parse_token_id(token_id: &str) -> StdResult<(u64, u64)> {
    let split = token_id.split('|').collect::<Vec<&str>>();
    if split.len() != 2 {
        return Err(StdError::generic_err(format!(
            "invalid token id `{}`: must be in the format {{serial}}|{{id}}",
            token_id
        )));
    }

    let id = u64::from_str(split[0]).map_err(|err| StdError::parse_err(type_name::<u64>(), err))?;
    let serial =
        u64::from_str(split[1]).map_err(|err| StdError::parse_err(type_name::<u64>(), err))?;

    Ok((id, serial))
}

/// The badge's id and serial are prepended to it's list of traits.
pub fn prepend_traits(mut metadata: Metadata, id: u64, serial: u64) -> Metadata {
    let mut traits = vec![
        Trait {
            display_type: None,
            trait_type: "id".to_string(),
            value: id.to_string(),
        },
        Trait {
            display_type: None,
            trait_type: "serial".to_string(),
            value: serial.to_string(),
        },
    ];

    traits.extend(metadata.attributes.unwrap_or_default().into_iter());

    metadata.attributes = Some(traits);
    metadata
}
