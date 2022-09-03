use cosmwasm_std::{StdError, VerificationError};
use thiserror::Error;

use badges::MintRule;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    FromHex(#[from] hex::FromHexError),

    #[error("{0}")]
    ParseReply(#[from] cw_utils::ParseReplyError),

    #[error("{0}")]
    Verification(#[from] VerificationError),

    #[error("invalid reply id {0}; must be 1")]
    InvalidReplyId(u64),

    #[error("signature verification failed")]
    InvalidSignature,

    #[error("unauthorized: sender is not badge manager")]
    NotManager,

    #[error("unauthorized: sender is not badge minter")]
    NotMinter,

    #[error("expecting the badge to be unavailable but it is available")]
    Available,

    #[error("badge minting deadline has been been exceeded")]
    Expired,

    #[error("badge max supply has been been exceeded")]
    SoldOut,

    #[error("key {key} already exists for badge {id}")]
    KeyExists {
        id: u64,
        key: String,
    },

    #[error("the provided key does not exist for badge {id}")]
    KeyDoesNotExist {
        id: u64,
    },

    #[error("user {user} has already claimed badge {id}")]
    AlreadyClaimed {
        id: u64,
        user: String,
    },

    #[error("unknown mint rule {found}, expecting by_minter|key|keys")]
    UnknownMintRule {
        found: String,
    },

    #[error("wrong mint rule: expected {expected}, found {found}")]
    WrongMintRule {
        expected: String,
        found: String,
    },
}

impl ContractError {
    pub fn key_exists(id: u64, key: impl Into<String>) -> Self {
        ContractError::KeyExists {
            id,
            key: key.into(),
        }
    }

    pub fn key_does_not_exist(id: u64) -> Self {
        ContractError::KeyDoesNotExist {
            id,
        }
    }

    pub fn already_claimed(id: u64, user: impl Into<String>) -> Self {
        ContractError::AlreadyClaimed {
            id,
            user: user.into(),
        }
    }

    pub fn unknown_mint_rule(found: impl Into<String>) -> Self {
        ContractError::UnknownMintRule {
            found: found.into(),
        }
    }

    pub fn wrong_mint_rule(expected: impl Into<String>, found: &MintRule) -> Self {
        ContractError::WrongMintRule {
            expected: expected.into(),
            found: found.to_string(),
        }
    }
}
