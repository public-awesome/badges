use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use badges::hub::{
    BadgeResponse, BadgesResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, KeyResponse,
    KeysResponse, OwnerResponse, OwnersResponse, QueryMsg,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);

    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(BadgeResponse), &out_dir);
    export_schema(&schema_for!(BadgesResponse), &out_dir);
    export_schema(&schema_for!(KeyResponse), &out_dir);
    export_schema(&schema_for!(KeysResponse), &out_dir);
    export_schema(&schema_for!(OwnerResponse), &out_dir);
    export_schema(&schema_for!(OwnersResponse), &out_dir);
}
