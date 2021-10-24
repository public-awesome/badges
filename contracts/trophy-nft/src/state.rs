use std::str::FromStr;
use std::string::ToString;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, BlockInfo, StdError};
use cw721::{Approval as Cw721Approval, Expiration};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};

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

/// Each NFT is represented by a 2-tuple: (trophy_id: u64, serial: u64)
///
/// A **trophy** is a collection of NFTs with the same metadata; trophy ID starts from 1 and goes up.
///
/// Each NFT is an **instance** of a trophy, identified by a serial number, which starts from 1 and
/// goes up.
///
/// For example, the 420th instance in the 69th trophy is identified by tuple (64, 420)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenId(u64, u64);

impl TokenId {
    pub fn new(trophy_id: u64, serial: u64) -> Self {
        Self(trophy_id, serial)
    }

    pub fn trophy_id(&self) -> u64 {
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

        let trophy_id_str = s_split[0];
        let trophy_id = u64::from_str(trophy_id_str)
            .map_err(|_| StdError::generic_err(format!("invalid batch id: {}", trophy_id_str)))?;

        let serial_str = s_split[1];
        let serial = u64::from_str(serial_str)
            .map_err(|_| StdError::generic_err(format!("invalid batch serial: {}", serial_str)))?;

        Ok(TokenId(trophy_id, serial))
    }
}

impl ToString for TokenId {
    fn to_string(&self) -> String {
        format!("{},{}", self.0, self.1)
    }
}

// We store token info in an indexed map indexed by owner address
pub struct TokenIndexes<'a> {
    // pk goes to second tuple element
    pub owner: MultiIndex<'a, (Addr, Vec<u8>), TokenInfo>,
}

// From cw-storage-plus docs:
//
// Note: this code is more or less boiler-plate, and needed for the internals. Do not try to
// customize this; just return a list of all indexes.
impl<'a> IndexList<TokenInfo> for TokenIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<TokenInfo>> + '_> {
        let v: Vec<&dyn Index<TokenInfo>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

/// A wrapper struct for easy handling of contract state
pub struct State<'a> {
    pub hub: Item<'a, Addr>,
    pub operators: Map<'a, (&'a Addr, &'a Addr), Expiration>,
    pub tokens: IndexedMap<'a, &'a str, TokenInfo, TokenIndexes<'a>>,
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
            hub: Item::new("hub"),
            operators: Map::new("operators"),
            tokens: IndexedMap::new("tokens", indexes),
            token_count: Item::new("token_count"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terra_trophies::testing::assert_generic_error_message;

    #[test]
    fn token_id_from_str() {
        let token_id = TokenId::from_str("69,420").unwrap();
        assert_eq!(token_id, TokenId::new(69, 420));

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
    fn token_id_to_string() {
        let token_id = TokenId::new(69, 420);
        let token_id_str: String = token_id.to_string();
        assert_eq!(token_id_str, "69,420".to_string());
    }
}
