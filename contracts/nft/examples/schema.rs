use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use badge_nft::msg::{AllNftInfoResponse, ExecuteMsg, InstantiateMsg, NftInfoResponse, QueryMsg};
use cw721::{
    ApprovalResponse, ApprovalsResponse, ContractInfoResponse, NumTokensResponse,
    OperatorsResponse, OwnerOfResponse, TokensResponse,
};
use cw721_base::msg::MinterResponse;
use sg721_base::msg::CollectionInfoResponse;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);

    export_schema(&schema_for!(ContractInfoResponse), &out_dir);
    export_schema(&schema_for!(NumTokensResponse), &out_dir);
    export_schema(&schema_for!(OwnerOfResponse), &out_dir);
    export_schema(&schema_for!(ApprovalResponse), &out_dir);
    export_schema(&schema_for!(ApprovalsResponse), &out_dir);
    export_schema(&schema_for!(OperatorsResponse), &out_dir);
    export_schema(&schema_for!(NftInfoResponse), &out_dir);
    export_schema(&schema_for!(AllNftInfoResponse), &out_dir);
    export_schema(&schema_for!(TokensResponse), &out_dir);

    export_schema(&schema_for!(MinterResponse), &out_dir);

    export_schema(&schema_for!(CollectionInfoResponse), &out_dir);
}
