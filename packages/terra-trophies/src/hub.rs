use cosmwasm_std::{Addr, Api, StdResult};
use cw721_metadata_onchain::Metadata;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TrophyInfo<T> {
    /// Address of the collection's creator, who has the authority to edit the collection
    pub creator: T,
    /// The collection's metadata
    pub metadata: Metadata,
}

impl From<TrophyInfo<Addr>> for TrophyInfo<String> {
    fn from(info: TrophyInfo<Addr>) -> Self {
        Self {
            creator: info.creator.to_string(),
            metadata: info.metadata,
        }
    }
}

impl TrophyInfo<String> {
    pub fn check(&self, api: &dyn Api) -> StdResult<TrophyInfo<Addr>> {
        Ok(TrophyInfo {
            creator: api.addr_validate(&self.creator)?,
            metadata: self.metadata.clone(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct InstantiateMsg {
    /// Code ID of the `trophy-nft` contract
    pub nft_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Create a new trophy with the specified metadata
    CreateTrophy(Metadata),
    /// Update metadata an existing trophy. Only the creator of the collection can call
    EditTrophy {
        trophy_id: u64,
        metadata: Metadata,
    },
    /// Mint new instances of a specified trophy to a list of addresses
    MintTrophy {
        trophy_id: u64,
        owners: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Query info about a trophy. Returns `TrophyInfo<String>`
    TrophyInfo {
        trophy_id: u64,
    },
}

pub mod helpers {
    use super::{QueryMsg, TrophyInfo};
    use cosmwasm_std::{to_binary, Addr, QuerierWrapper, QueryRequest, StdResult, WasmQuery};
    use cw721_metadata_onchain::Metadata;

    pub fn query_trophy_metadata(
        querier: &QuerierWrapper,
        hub_addr: &Addr,
        trophy_id: u64,
    ) -> StdResult<Metadata> {
        let trophy_info: TrophyInfo<String> =
            querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: hub_addr.to_string(),
                msg: to_binary(&QueryMsg::TrophyInfo {
                    trophy_id,
                })?,
            }))?;
        Ok(trophy_info.metadata)
    }
}
