use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{Participant, AlloInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub fund_denom: String,
    pub reward_token: String,
    pub vesting: String,
    pub whitelist_merkle_root: String,

    pub exchange_rate: Uint128,
    pub private_start_time: u64,
    pub public_start_time: u64,
    pub presale_period: u64,

    pub total_rewards_amount: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct MigrateMsg {}

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
    Deposit {
        allo_info: AlloInfo,
        proof: Vec<String>,
    },
    DepositPrivateSale {
        allo_info: AlloInfo,
        proof: Vec<String>,
    },
    WithdrawFunds {
        receiver: String,
    },
    WithdrawUnsoldToken {
        receiver: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    ParticipantsCount {},
    GetSaleStatus {},
    GetParticipants {
        page: u64,
        limit: u64,
    },
    GetParticipant {
        user: String,
    },
    PresaleInfo {}
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
    pub private_sold_amount: Uint128,
    pub public_sold_amount: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PresaleInfoResponse {
    // owner
    pub owner: String,
    // Accuracy
    pub accuracy: Uint128,
    // Exchange rate
    pub exchange_rate: Uint128,
    // Presale Period.
    pub presale_period: u64,
    // Public Presale Start Time.
    pub public_start_time: u64,
    // Private Presale Start Time.
    pub private_start_time: u64,
    // Accuracy
    pub total_rewards_amount: Uint128,
}