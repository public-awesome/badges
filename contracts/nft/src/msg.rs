use cosmwasm_std::Empty;

pub type Extension = Option<Empty>;
pub type InstantiateMsg = sg721::InstantiateMsg;
pub type ExecuteMsg = sg721::ExecuteMsg<Extension>;
pub type QueryMsg = sg721_base::msg::QueryMsg;
