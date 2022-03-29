use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    // Owner address
    pub owner: CanonicalAddr,

    /************** Address Infos *************/
    // Token for fundraise.
    pub fund_token: CanonicalAddr,
    // Token for distribution.
    pub reward_token: CanonicalAddr,
    // Vesting Contract.
    pub vesting: CanonicalAddr,
    // Whitelist Merkle Root.
    pub whitelist_merkle_root: String,

    /************** Presale Params *************/
    // Fixed rate between fundToken vs rewardToken = reward / fund * ACCURACY.
    pub exchange_rate: u64,
    // Presale Period.
    pub presale_period: u64,
    // Public Presale Start Time.
    pub public_start_time: u64,
    // Private Presale Start Time.
    pub private_start_time: u64,
    // Token amount to distribute.
    pub distribution_amount: u64,

    /************** Status Info *************/
    // Reward token amount sold by private sale
    pub private_sold_amount: u64,
    // Reward token amount sold by public sale
    pub public_sold_amount: u64,
    // Participants address list
    pub userlist: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Participant {
    // Fund token amount by participant.
    pub fund_balance: u64,
    // Reward token amount need to be vested.
    pub reward_balance: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AlloInfo {
    // Max allocation for this user in public presale
    pub public_allocation: u64,
    // Max allocation for this user in private presale
    pub private_allocation: u64,
}

pub const STATE: Item<State> = Item::new("state");

pub const PRIVATE_SOLD_FUNDS: Map<String, u64> = Map::new("private_sold_funds");

pub const PARTICIPANTS: Map<String, Participant> = Map::new("participants");

pub const ACCURACY: u64 = 100000000;
