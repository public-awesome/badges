use cosmwasm_std::Empty;
use sg_metadata::Metadata;

pub type Extension = Option<Empty>;

pub type InstantiateMsg = sg721::InstantiateMsg;
pub type ExecuteMsg = sg721::ExecuteMsg<Extension>;
pub type QueryMsg = sg721_base::msg::QueryMsg;

pub type NftInfoResponse = cw721::NftInfoResponse<Metadata>;
pub type AllNftInfoResponse = cw721::AllNftInfoResponse<Metadata>;
