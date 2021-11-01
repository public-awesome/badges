use cosmwasm_std::{Addr, Api, StdResult};
use cw721::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::metadata::Metadata;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MintRule<T> {
    /// Trophy instances can be minted by a designated minter, which can either be an an externally
    /// owned account that does the minting manually, or a smart contract that implements custom
    /// minting logic
    ///
    /// To use this rule, provide the minter's address
    ByMinter(T),
    /// Trophy instances can be minted by anyone who knows a specific private key. When creating the
    /// trophy, the creator generates a public/private key pair, and informs Hub of the public key.
    /// When minting, a user signs a message containing his address by the private key. Hub mints
    /// the NFT if signature is valid
    ///
    /// To use this rule, supply the base64-encoded public key
    BySignature(String),
}

impl From<MintRule<Addr>> for MintRule<String> {
    fn from(rule: MintRule<Addr>) -> Self {
        match rule {
            MintRule::ByMinter(minter) => MintRule::ByMinter(minter.to_string()),
            MintRule::BySignature(pubkey) => MintRule::BySignature(pubkey),
        }
    }
}

impl MintRule<String> {
    pub fn check(&self, api: &dyn Api) -> StdResult<MintRule<Addr>> {
        Ok(match self {
            MintRule::ByMinter(minter) => MintRule::ByMinter(api.addr_validate(minter)?),
            MintRule::BySignature(pubkey) => MintRule::BySignature(pubkey.clone()),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TrophyInfo<T> {
    /// Address of the collection's creator, who has the authority to edit the collection
    pub creator: T,
    /// How instances of this trophy are to be minted. See the documentation of `MintRule`
    pub rule: MintRule<T>,
    /// The collection's metadata
    pub metadata: Metadata,
    /// The deadline before which instances of this trophy can be minted
    pub expiry: Option<Expiration>,
    /// The maximum number of trophy instances can be minted
    pub max_supply: Option<u64>,
    /// The current number of this trophy's instances
    pub current_supply: u64,
}

impl From<TrophyInfo<Addr>> for TrophyInfo<String> {
    fn from(info: TrophyInfo<Addr>) -> Self {
        Self {
            creator: info.creator.to_string(),
            rule: info.rule.into(),
            metadata: info.metadata,
            expiry: info.expiry,
            max_supply: info.max_supply,
            current_supply: info.current_supply,
        }
    }
}

impl TrophyInfo<String> {
    pub fn check(&self, api: &dyn Api) -> StdResult<TrophyInfo<Addr>> {
        Ok(TrophyInfo {
            creator: api.addr_validate(&self.creator)?,
            rule: self.rule.check(api)?,
            metadata: self.metadata.clone(),
            expiry: self.expiry,
            max_supply: self.max_supply,
            current_supply: self.current_supply,
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
    CreateTrophy {
        /// Defines the rules on how instances of the trophy shall be minted. See docs of `MintRule`
        rule: MintRule<String>,
        /// Metadata of this trophy
        metadata: Metadata,
        /// The deadline before which instances of this trophy can be minted
        expiry: Option<Expiration>,
        /// The maximum number of trophy instances can be minted
        max_supply: Option<u64>,
    },
    /// Update metadata an existing trophy. Only the creator of the collection can call
    EditTrophy {
        /// Identifier of the trophy
        trophy_id: u64,
        /// The new metadata for the trophy
        metadata: Metadata,
    },
    /// Mint new instances of a specified trophy to a list of addresses. Called only if the trophy's
    /// minting rule is set to `ByOwner` and if caller if owner
    MintByMinter {
        /// Idnetifier of the trophy
        trophy_id: u64,
        /// A list of owners to receive instances of the trophy
        owners: Vec<String>,
    },
    /// Mint a new instance of the trophy by submitting a signature. The message should be the
    /// caller's address, and the private key is the one created for this trophy. Called only if the
    /// trophy's minting rule is set to `BySignature`
    MintBySignature {
        /// Idnetifier of the trophy
        trophy_id: u64,
        /// Base64-encoded signature signed by the private key; the content is the caller's address
        signature: String,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Total number of existing trophies. Returns `ContractInfoResponse`
    ContractInfo {},
    /// Info about a trophy. Returns `TrophyInfo<String>`
    TrophyInfo {
        trophy_id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ContractInfoResponse {
    pub nft: String,
    pub trophy_count: u64,
}

pub mod helpers {
    use super::{Metadata, QueryMsg, TrophyInfo};
    use cosmwasm_std::{to_binary, Addr, QuerierWrapper, QueryRequest, StdResult, WasmQuery};

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
