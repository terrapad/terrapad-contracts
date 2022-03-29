use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{Participant, AlloInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub fund_token: String,
    pub reward_token: String,
    pub vesting: String,
    pub whitelist_merkle_root: String,

    pub exchange_rate: u64,
    pub private_start_time: u64,
    pub public_start_time: u64,
    pub presale_period: u64,
    pub distribution_amount: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    TransferOwnerShip {
        new_owner: String
    },
    SetMerkleRoot {
        /// MerkleRoot is hex-encoded merkle root.
        merkle_root: String,
    },
    UpdatePresaleInfo {
        new_private_start_time: u64,
        new_public_start_time: u64,
        new_presale_period: u64
    },
    Receive(Cw20ReceiveMsg),
    WithdrawFunds {
        receiver: String,
    },
    WithdrawUnsoldToken {
        receiver: String,
    },
    StartVesting {},
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Deposit {
        allo_info: AlloInfo,
        proof: Vec<String>,
    },
    DepositPrivateSale {
        allo_info: AlloInfo,
        proof: Vec<String>,
    },
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    ParticipantsCount {},
    GetParticipants {
        page: u64,
        limit: u64,
    },
    GetParticipant {
        user: String,
    },
    GetSaleStatus {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ParticipantsCountResponse {
    pub count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetParticipantsResponse {
    pub participants: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetParticipantResponse {
    pub data: Participant,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetSaleStatusResponse {
    pub private_sold_amount: u64,
    pub public_sold_amount: u64
}
