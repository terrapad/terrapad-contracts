use std::env::current_dir;
use std::fs::create_dir_all;

use locking::msg::{
    InstantiateMsg, QueryMsg, ExecuteMsg, ConfigResponse, LockInfoResponse, LockedAccountsResponse, Cw20HookMsg
};
use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(Cw20HookMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(LockInfoResponse), &out_dir);
    export_schema(&schema_for!(LockedAccountsResponse), &out_dir);
}
