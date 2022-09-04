use std::any::type_name;
use std::ops::Deref;
use std::str::FromStr;

use cosmwasm_std::{Deps, Empty, Env, StdError, StdResult};
use cw721::{AllNftInfoResponse, Cw721Query, NftInfoResponse};
use sg_metadata::{Metadata, Trait};

use badges::Badge;

pub type Extension = Option<Empty>;
pub type InstantiateMsg = sg721::InstantiateMsg;
pub type ExecuteMsg = sg721::ExecuteMsg<Extension>;
pub type QueryMsg = sg721_base::msg::QueryMsg;

#[derive(Default)]
pub struct NftContract<'a>(sg721_base::Sg721Contract<'a, Extension>);

// Implement the Deref trait, so that we can access the parent contract's methods without explicitly
// referencing it as `self.0`.
impl<'a> Deref for NftContract<'a> {
    type Target = sg721_base::Sg721Contract<'a, Extension>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> NftContract<'a> {
    /// Overrides vanilla cw721's `nft_info` method
    pub fn nft_info(
        &self,
        deps: Deps,
        token_id: impl ToString,
    ) -> StdResult<NftInfoResponse<Metadata>> {
        let (id, serial) = parse_token_id(&token_id.to_string())?;
        let uri = uri(id, serial);
        let metadata = self.query_metadata(deps, id)?;
        Ok(NftInfoResponse {
            token_uri: Some(uri),
            extension: prepend_traits(metadata, id, serial),
        })
    }

    /// Overrides vanilla cw721's `all_nft_info` method
    pub fn all_nft_info(
        &self,
        deps: Deps,
        env: Env,
        token_id: impl ToString,
        include_expired: Option<bool>,
    ) -> StdResult<AllNftInfoResponse<Metadata>> {
        let access = self.parent.owner_of(
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
    fn query_metadata(&self, deps: Deps, id: u64) -> StdResult<Metadata> {
        let minter = self.parent.minter(deps)?;
        let badge: Badge<String> = deps.querier.query_wasm_smart(
            &minter.minter,
            &badges::hub::QueryMsg::Badge {
                id,
            },
        )?;
        Ok(badge.metadata)
    }
}

/// URL of an API serving the metadata of the NFT.
///
/// A benefit of dynamically generating the URL instead of saving it in the contract storage is that
/// if I later want to update the URL, I only need to change this one function, instead of changing
/// every token's data.
pub fn uri(id: u64, serial: u64) -> String {
    format!("https://badges-api.larry.engineer/metadata?id={}&serial={}", id, serial)
}

/// Split a token id into badge id and serial number.
/// The token id must be in the format `{u64}|{u64}`, where the 1st number is id and 2nd is serial.
pub fn parse_token_id(token_id: &str) -> StdResult<(u64, u64)> {
    let split = token_id.split('|').collect::<Vec<&str>>();
    if split.len() != 2 {
        return Err(StdError::generic_err(
            format!("invalid token id `{}`: must be in the format {{serial}}|{{id}}", token_id),
        ));
    }

    let id = u64::from_str(split[0])
        .map_err(|err| StdError::parse_err(type_name::<u64>(), err))?;
    let serial = u64::from_str(split[1])
        .map_err(|err| StdError::parse_err(type_name::<u64>(), err))?;

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
