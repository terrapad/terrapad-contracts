use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    /************** Vesting Params *************/
    // Start time of vesting
    pub start_time: u64,
    // Intervals that the release happens. Every interval, releaseRate of tokens are released.
    pub release_interval: u64,
    // Release percent in each withdrawing interval
    pub release_rate: u64,
    // Percent of tokens initially unlocked
    pub initial_unlock: u64,
    // Period before release vesting starts, also it unlocks initialUnlock reward tokens.
    pub lock_period: u64,
    // Period to release all reward token, after lockPeriod + vestingPeriod it releases 100% of reward tokens.
    pub vesting_period: u64,
    // Reward token of the project.
    pub reward_token: CanonicalAddr,

    /************** Status Info *************/
    // Owner address(presale or the vesting runner)
    pub owner: CanonicalAddr,
    // Worker address(presale or the vesting runner)
    pub worker: CanonicalAddr,
    // Sum of all user's vesting amount
    pub total_vesting_amount: u64,
    // Participants address list
    pub userlist: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserInfo {
    // Total amount of tokens to be vested.
    pub total_amount: u64,
    // The amount that has been withdrawn.
    pub withrawn_amount: u64,
}

pub const STATE: Item<State> = Item::new("state");

pub const RECIPIENTS: Map<String, UserInfo> = Map::new("recipients");

pub const ACCURACY: u64 = 1000;
