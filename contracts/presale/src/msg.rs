use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{Participant};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub fund_token: String,
    pub reward_token: String,
    pub vesting: String,
    pub whitelist: String,

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
    UpdatePresaleInfo {
        new_private_start_time: u64,
        new_public_start_time: u64,
        new_presale_period: u64
    },
    Deposit {
        amount: u64,
    },
    DepositPrivateSale {
        amount: u64,
    },
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
