use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use presale::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ParticipantsCountResponse, GetSaleStatusResponse, GetParticipantResponse, GetParticipantsResponse};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(ParticipantsCountResponse), &out_dir);
    export_schema(&schema_for!(GetSaleStatusResponse), &out_dir);
    export_schema(&schema_for!(GetParticipantResponse), &out_dir);
    export_schema(&schema_for!(GetParticipantsResponse), &out_dir);
}
