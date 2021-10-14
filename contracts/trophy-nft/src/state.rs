use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, BlockInfo, StdError};
use cw721::{Approval as Cw721Approval, ContractInfoResponse, Expiration};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex, U64Key};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BatchInfo {
    /// Identifies the asset to which this NFT represents
    pub name: String,
    /// Describes the asset to which this NFT represents
    pub description: String,
    /// URI pointing to a resource with mime type image/* representing the asset to which this NFT represents
    ///
    /// Unlike the vanilla CW721 implementation, we require each batch must have an image. Seriously,
    /// why would you mint an NFT when you don't even have an image?
    pub image: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo {
    /// The owner of the newly minted NFT
    pub owner: Addr,
    /// Approvals are stored here, as we clear them all upon transfer and cannot accumulate much
    pub approvals: Vec<Approval>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Approval {
    /// Account that can transfer/send the token
    pub spender: Addr,
    /// When the Approval expires (maybe Expiration::never)
    pub expires: Expiration,
}

impl Approval {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires.is_expired(block)
    }

    pub fn humanize(&self) -> Cw721Approval {
        Cw721Approval {
            spender: self.spender.to_string(),
            expires: self.expires,
        }
    }
}

/// Each NFT is represented by a 2-tuple: (batch_id: u64, serial: u64)
///
/// A batch is a collection of NFTs with the same name, description, and image; batch ID starts from
/// 1 and goes up
///
/// Each NFT in a batch is identified by a serial number, which starts from 1 and goes up
/// For example, the 420th NFT in the 69th batch is identified by tuple (64, 420)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenId(pub u64, pub u64);

impl TokenId {
    pub fn batch_id(&self) -> u64 {
        self.0
    }

    pub fn serial(&self) -> u64 {
        self.1
    }
}

impl FromStr for TokenId {
    type Err = StdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s_split: Vec<&str> = s.split(',').collect();
        if s_split.len() != 2 {
            return Err(StdError::generic_err(format!("invalid token id: {}", s)));
        }

        let batch_id_str = s_split[0];
        let batch_id = u64::from_str(batch_id_str)
            .map_err(|_| StdError::generic_err(format!("invalid batch id: {}", batch_id_str)))?;

        let serial_str = s_split[1];
        let serial = u64::from_str(serial_str)
            .map_err(|_| StdError::generic_err(format!("invalid batch serial: {}", serial_str)))?;

        Ok(TokenId(batch_id, serial))
    }
}

impl From<TokenId> for String {
    fn from(token_id: TokenId) -> String {
        format!("{},{}", token_id.0, token_id.1)
    }
}

impl From<TokenId> for (U64Key, U64Key) {
    fn from(token_id: TokenId) -> (U64Key, U64Key) {
        (token_id.0.into(), token_id.1.into())
    }
}

//  We store token info in an indexed map indexed by two attributes:
// - owner address
// - batch id
pub struct TokenIndexes<'a> {
    // pk goes to second tuple element
    pub owner: MultiIndex<'a, (Addr, Vec<u8>), TokenInfo>,
}

// From cw-storage-plus docs:
//  Note: this code is more or less boiler-plate, and needed for the internals. Do not try to
// customize this; just return a list of all indexes.
impl<'a> IndexList<TokenInfo> for TokenIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<TokenInfo>> + '_> {
        let v: Vec<&dyn Index<TokenInfo>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

/// A wrapper struct for easy handling of contract state
pub struct State<'a> {
    pub contract_info: Item<'a, ContractInfoResponse>,
    pub minter: Item<'a, Addr>,
    pub operators: Map<'a, (&'a Addr, &'a Addr), Expiration>,
    pub batches: Map<'a, U64Key, BatchInfo>,
    pub batch_count: Item<'a, u64>,
    pub tokens: IndexedMap<'a, (U64Key, U64Key), TokenInfo, TokenIndexes<'a>>,
    pub token_count: Item<'a, u64>,
}

impl<'a> Default for State<'a> {
    fn default() -> Self {
        let indexes = TokenIndexes {
            owner: MultiIndex::new(
                |d: &TokenInfo, k: Vec<u8>| (d.owner.clone(), k),
                "tokens",
                "tokens__owner",
            ),
        };
        Self {
            contract_info: Item::new("contract_info"),
            minter: Item::new("minter"),
            operators: Map::new("operators"),
            batches: Map::new("batches"),
            batch_count: Item::new("batch_count"),
            tokens: IndexedMap::new("tokens", indexes),
            token_count: Item::new("token_count"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::StdResult;

    #[test]
    fn token_id_from_str() {
        let token_id = TokenId::from_str("69,420").unwrap();
        assert_eq!(token_id, TokenId(69, 420));

        let result = TokenId::from_str("69");
        assert_generic_error_message(result, "invalid token id: 69");

        let result = TokenId::from_str("69,420,12345");
        assert_generic_error_message(result, "invalid token id: 69,420,12345");

        let result = TokenId::from_str("ngmi");
        assert_generic_error_message(result, "invalid token id: ngmi");

        let result = TokenId::from_str("ngmi,420");
        assert_generic_error_message(result, "invalid batch id: ngmi");

        let result = TokenId::from_str("69,ngmi");
        assert_generic_error_message(result, "invalid batch serial: ngmi");
    }

    #[test]
    fn token_id_into_string() {
        let token_id = TokenId(69, 420);
        let token_id_str: String = token_id.into();
        assert_eq!(token_id_str, "69,420".to_string());
    }

    pub fn assert_generic_error_message<T>(result: StdResult<T>, expected_msg: &str) {
        match result {
            Err(StdError::GenericErr {
                msg,
                ..
            }) => assert_eq!(msg, expected_msg),
            Err(other_err) => panic!("unexpected error: {:?}", other_err),
            Ok(_) => panic!("expected error but ok"),
        }
    }
}
