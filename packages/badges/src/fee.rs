use cosmwasm_std::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Describes the rate of fees charged for storing data on-chain.
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct FeeRate {
    /// The fee rate, in ustars per byte, for storing metadata on-chain
    pub metadata: Decimal,

    /// The fee rate, in ustars per byte, for storing claim keys on-chain
    pub key: Decimal,
}
