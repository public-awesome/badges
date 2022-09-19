use cosmwasm_std::Empty;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg_metadata::Metadata;

pub type Extension = Option<Empty>;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct InstantiateMsg {
    /// Address of the Badges Hub contract
    pub hub: String,
    /// URL of an API that serves the Badges metadata.
    /// The full URL will be `${api_url}?id=${id}&serial=${serial}`
    pub api_url: String,
    /// SG-721 collection info
    pub collection_info: sg721::CollectionInfo<sg721::RoyaltyInfoResponse>,
}

// message types
pub type ExecuteMsg = sg721::ExecuteMsg<Extension>;
pub type QueryMsg = sg721_base::msg::QueryMsg;

// response types
pub type ContractInfoResponse = cw721::ContractInfoResponse;
pub type NumTokensResponse = cw721::NumTokensResponse;
pub type OwnerOfResponse = cw721::OwnerOfResponse;
pub type TokensResponse = cw721::TokensResponse;
pub type ApprovalResponse = cw721::ApprovalResponse;
pub type ApprovalsResponse = cw721::ApprovalsResponse;
pub type OperatorsResponse = cw721::OperatorsResponse;
pub type NftInfoResponse = cw721::NftInfoResponse<Metadata>;
pub type AllNftInfoResponse = cw721::AllNftInfoResponse<Metadata>;
pub type MinterResponse = cw721_base::MinterResponse;
pub type CollectionInfoResponse = sg721_base::msg::CollectionInfoResponse;
