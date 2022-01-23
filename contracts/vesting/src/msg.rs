use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::UserInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub reward_token: String,
    pub release_interval: u64,
    pub release_rate: u64,
    pub initial_unlock: u64,
    pub lock_period: u64,
    pub vesting_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    TransferOwnerShip {
        new_owner: String
    },
    UpdateRecipient {
        recp: String,
        amount: u64
    },
    SetStartTime {
        new_start_time: u64
    },
    Withdraw {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    UsersCount {},
    GetUsers {
        page: u64,
        limit: u64,
    },
    GetUser {
        user: String,
    },
    Vested {
        user: String,
    },
    Locked {
        user: String,
    },
    Withdrawable {
        user: String,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UsersCountResponse {
    pub count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetUsersResponse {
    pub users: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetUserResponse {
    pub data: UserInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AmountResponse {
    pub amount: u64,
}
